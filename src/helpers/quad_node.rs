use cgmath::num_traits::Float;

pub struct QuadNode<T: Float> {
    pub u_min: T,
    pub u_max: T,

    pub v_min: T,
    pub v_max: T,

    pub level: u32,

    pub children: Option<Box<[QuadNode<T>; 4]>>,
}

impl<T: Float> QuadNode<T> {
    pub fn new() -> Self {
        Self {
            u_min: -T::one(),
            u_max: T::one(),

            v_min: -T::one(),
            v_max: T::one(),

            level: 0,

            children: None,
        }
    }

    pub fn split(&mut self) {
        if self.children.is_some() {
            return;
        }
        let two = T::one() + T::one();
        let um = (self.u_min + self.u_max) / two;
        let vm = (self.v_min + self.v_max) / two;

        self.children = Some(Box::new([
            QuadNode {
                u_min: self.u_min,
                u_max: um,
                v_min: self.v_min,
                v_max: vm,
                level: self.level + 1,
                children: None,
            },
            QuadNode {
                u_min: um,
                u_max: self.u_max,
                v_min: self.v_min,
                v_max: vm,
                level: self.level + 1,
                children: None,
            },
            QuadNode {
                u_min: self.u_min,
                u_max: um,
                v_min: vm,
                v_max: self.v_max,
                level: self.level + 1,
                children: None,
            },
            QuadNode {
                u_min: um,
                u_max: self.u_max,
                v_min: vm,
                v_max: self.v_max,
                level: self.level + 1,
                children: None,
            },
        ]));
    }

    pub fn subdivide_to(&mut self, max_level: u32) {
        if self.level >= max_level {
            return;
        }

        self.split();

        if let Some(children) = self.children.as_mut() {
            for child in children.iter_mut() {
                child.subdivide_to(max_level);
            }
        }
    }
}
