use std::collections::LinkedList;

fn bit_separate(mut n: usize) -> usize {
    // max 2^16
    n = (n | (n << 8)) & 0x00ff00ff;
    n = (n | (n << 4)) & 0x0f0f0f0f;
    n = (n | (n << 2)) & 0x33333333;
    (n | (n << 1)) & 0x55555555
}

pub struct QuadTreeGrid<T> {
    level: usize,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    tree: Vec<LinkedList<T>>,
}

impl<T> QuadTreeGrid<T> {
    pub fn new(level: usize, x: f64, y: f64, width: f64, height: f64) -> Self {
        let bias = (4usize.pow(level as u32) - 1) / 3;
        Self {
            level: level,
            x: x,
            y: y,
            width: width,
            height: height,
            tree: std::iter::repeat_with(|| LinkedList::new())
                .take(bias + 4usize.pow(level as u32))
                .collect(),
        }
    }

    fn get_grid_position(&self, x: f64, y: f64) -> (usize, usize) {
        let grid_size = 1 << self.level;
        let grid_width = self.width / grid_size as f64;
        let grid_height = self.height / grid_size as f64;
        let x = ((x - self.x) / grid_width) as usize;
        let x = if x < grid_size { x } else { grid_size - 1 };
        let y = ((y - self.y) / grid_height) as usize;
        let y = if y < grid_size { y } else { grid_size - 1 };
        (x, y)
    }

    fn get_morton_order(&self, x: f64, y: f64) -> usize {
        let (x, y) = self.get_grid_position(x, y);
        bit_separate(x) | (bit_separate(y) << 1)
    }

    fn get_index(&self, x: f64, y: f64, width: f64, height: f64) -> usize {
        let left_top = self.get_morton_order(x, y);
        let right_bottom = self.get_morton_order(x + width, y + height);
        let mut xor = left_top ^ right_bottom;
        let mut level = self.level;
        while xor != 0 {
            level -= 1;
            xor = xor >> 2;
        }
        let space = right_bottom >> ((self.level - level) * 2);
        let bias = (4usize.pow(level as u32) - 1) / 3;
        space + bias
    }

    pub fn add_entity(&mut self, x: f64, y: f64, width: f64, height: f64, entity: T) {
        let index = self.get_index(x, y, width, height);
        self.tree[index].push_front(entity);
    }

    pub fn iter_entity_pair(&mut self, mut f: impl FnMut(&T, &T)) {
        self._iter_entity_pair(&mut f, 0, 0, &mut LinkedList::new());
    }

    fn _iter_entity_pair(
        &mut self,
        f: &mut dyn FnMut(&T, &T),
        level: usize,
        index: usize,
        stack: &mut LinkedList<T>,
    ) {
        let bias = (4usize.pow(level as u32) - 1) / 3;
        let tree_ptr = index + bias;

        let target_list = &mut self.tree[tree_ptr];
        let mut outer_iter = target_list.iter();
        loop {
            let mut inner_iter = outer_iter.clone();
            if let Some(target) = inner_iter.next() {
                for succ in inner_iter {
                    f(target, succ);
                }
                for stacked in stack.iter() {
                    f(target, stacked);
                }
            } else {
                break;
            }
            outer_iter.next();
        }

        if level < self.level {
            let len = target_list.len();
            stack.append(target_list);
            for i in 0..4 {
                self._iter_entity_pair(f, level + 1, index * 4 + i, stack);
            }
            stack.split_off(stack.len() - len);
        } else {
            target_list.clear();
        }
    }
}
