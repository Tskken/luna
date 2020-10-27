#[derive(Debug, Copy, Clone)]
pub struct Rectangle<T> {
    position: [T; 2],
    wh: [T; 2],
}

impl<T> Rectangle<T> 
where T: Copy
{
    pub fn new(x: T, y: T, w: T, h: T) -> Rectangle<T> {
        Rectangle {
            position: [x, y],
            wh: [w, h],
        }
    }

    pub fn x(self) -> T {
        self.position[0]
    }
    pub fn y(self) -> T {
        self.position[1]
    }

    pub fn w(self) -> T {
        self.wh[0]
    }
    pub fn h(self) -> T {
        self.wh[1]
    }
}