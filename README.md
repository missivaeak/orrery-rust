# Orrery

## References

https://www.youtube.com/watch?v=sLqXFF8mlEU&list=PLFt_AvWsXl0dT82XMtKATYPcVIhpu2fh6
https://www.youtube.com/watch?v=lThxbFvbRew
https://www.youtube.com/watch?v=QN39W020LqU
https://www.youtube.com/watch?v=r2CeFOavfdM
https://www.youtube.com/watch?v=HIYs7Hoq2yQ
https://github.com/HolmanDev/LOD-Planets-in-Unity/blocurl/master/Project/LOD-Planets/Assets/Scripts/Planet.cs

For a cube sphere, I would avoid making `Face` store a quadtree at all. Instead, make `Face` a *pure generator*. Given a camera position, it deterministically traverses the quadtree, decides which nodes should exist, and immediately emits mesh data. This keeps the system stateless and avoids synchronization issues.

A design like this scales well:

```rust
use glam::{Vec3, Vec2};

pub struct Face {
    /// Unit normal of this cube face.
    pub normal: Vec3,

    /// Local axes spanning the face.
    pub axis_u: Vec3,
    pub axis_v: Vec3,

    /// Radius of the planet.
    pub radius: f32,

    /// Maximum subdivision depth.
    pub max_depth: u32,
}

pub struct MeshData {
    pub vertices: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub indices: Vec<u32>,
}
```

The update function is simply

```rust
impl Face {
    pub fn update(&self, camera_pos: Vec3) -> MeshData {
        let mut mesh = MeshData {
            vertices: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new(),
        };

        self.build_node(
            camera_pos,
            Vec2::ZERO,
            Vec2::ONE,
            0,
            &mut mesh,
        );

        mesh
    }
}
```

No state is retained between frames.

---

## Recursive subdivision

Each node is represented only by:

* UV min/max
* depth

```rust
impl Face {
    fn build_node(
        &self,
        camera: Vec3,
        min: Vec2,
        max: Vec2,
        depth: u32,
        mesh: &mut MeshData,
    ) {
        let center = (min + max) * 0.5;

        let world_center = self.project(center);

        let distance = camera.distance(world_center);

        let size = (max.x - min.x) * self.radius;

        if depth < self.max_depth && self.should_split(distance, size) {
            let mid = center;

            self.build_node(camera, min, mid, depth + 1, mesh);

            self.build_node(
                camera,
                Vec2::new(mid.x, min.y),
                Vec2::new(max.x, mid.y),
                depth + 1,
                mesh,
            );

            self.build_node(
                camera,
                Vec2::new(min.x, mid.y),
                Vec2::new(mid.x, max.y),
                depth + 1,
                mesh,
            );

            self.build_node(camera, mid, max, depth + 1, mesh);
        } else {
            self.emit_patch(min, max, mesh);
        }
    }
}
```

---

## LOD criterion

A simple heuristic is

```rust
fn should_split(&self, distance: f32, size: f32) -> bool {
    distance < size * 8.0
}
```

A more stable version estimates screen-space error:

```text
screen_error = patch_world_size / distance
```

Split while

```text
screen_error > threshold
```

This gives nearly constant visual quality regardless of altitude.

---

## Cube → sphere projection

```rust
impl Face {
    fn project(&self, uv: Vec2) -> Vec3 {
        let p =
            self.normal +
            (uv.x * 2.0 - 1.0) * self.axis_u +
            (uv.y * 2.0 - 1.0) * self.axis_v;

        p.normalize() * self.radius
    }
}
```

For example

```text
+X face

normal = (1,0,0)
axis_u = (0,0,-1)
axis_v = (0,1,0)
```

---

## Emitting a leaf patch

For each leaf node you tessellate into a fixed-resolution grid.

```rust
fn emit_patch(
    &self,
    min: Vec2,
    max: Vec2,
    mesh: &mut MeshData,
) {
    const RES: usize = 16;

    let base = mesh.vertices.len() as u32;

    for y in 0..=RES {
        for x in 0..=RES {

            let u = min.x + (max.x - min.x) * x as f32 / RES as f32;
            let v = min.y + (max.y - min.y) * y as f32 / RES as f32;

            let p = self.project(Vec2::new(u, v));

            mesh.vertices.push(p);
            mesh.normals.push(p.normalize());
        }
    }

    let stride = RES + 1;

    for y in 0..RES {
        for x in 0..RES {

            let i0 = base + (y * stride + x) as u32;
            let i1 = i0 + 1;
            let i2 = i0 + stride as u32;
            let i3 = i2 + 1;

            mesh.indices.extend([
                i0, i2, i1,
                i1, i2, i3,
            ]);
        }
    }
}
```

---

## Why this stateless approach works well

Every frame:

```
camera
    ↓
Face::update()
    ↓
recursive subdivision
    ↓
leaf patches
    ↓
mesh
```

Nothing from the previous frame is required. The tree is implicit in the recursion, making it:

* deterministic,
* thread-safe,
* easy to parallelize (each of the six faces can be generated independently),
* free of stale quadtree nodes that need updating or pruning.

If generation becomes expensive at very high LODs, you can later introduce caching of generated leaf meshes or parallelize the recursion with a work-stealing library like `rayon`, while keeping the public API (`Face::update(camera_pos) -> MeshData`) unchanged.

