
pub trait Array<T> {
    fn from_fn(f: |uint| -> T) -> Self;
    fn from_iter<I: Iterator<T>>(mut iter: I) -> Self {
        Array::from_fn(|_| iter.next().unwrap())
    }
}

impl<T> Array<T> for [T, ..2] {
    fn from_fn(f: |uint| -> T) -> [T, ..2] {
        [f(0), f(1)]
    }
}

pub trait Array2<T> {
    fn map<U>(self, f: |T| -> U) -> [U, ..2];
    fn x(self) -> T;
    fn y(self) -> T;
}

impl<T: Copy> Array2<T> for [T, ..2] {
    fn map<U>(self, f: |T| -> U) -> [U, ..2] {
        let [a, b] = self;
        [f(a), f(b)]
    }
    fn x(self) -> T { self[0] }
    fn y(self) -> T { self[1] }
}

impl<T> Array<T> for [T, ..3] {
    fn from_fn(f: |uint| -> T) -> [T, ..3] {
        [f(0), f(1), f(2)]
    }
}

pub trait Array3<T> {
    fn map<U>(self, f: |T| -> U) -> [U, ..3];
    fn x(self) -> T;
    fn y(self) -> T;
    fn z(self) -> T;
}

impl<T: Copy> Array3<T> for [T, ..3] {
    fn map<U>(self, f: |T| -> U) -> [U, ..3] {
        let [a, b, c] = self;
        [f(a), f(b), f(c)]
    }
    fn x(self) -> T { self[0] }
    fn y(self) -> T { self[1] }
    fn z(self) -> T { self[2] }
}

impl<T> Array<T> for [T, ..4] {
    fn from_fn(f: |uint| -> T) -> [T, ..4] {
        [f(0), f(1), f(2), f(3)]
    }
}

pub trait Array4<T> {
    fn map<U>(self, f: |T| -> U) -> [U, ..4];
    fn x(self) -> T;
    fn y(self) -> T;
    fn z(self) -> T;
    fn w(self) -> T;
}

impl<T: Copy> Array4<T> for [T, ..4] {
    fn map<U>(self, f: |T| -> U) -> [U, ..4] {
        let [a, b, c, d] = self;
        [f(a), f(b), f(c), f(d)]
    }
    fn x(self) -> T { self[0] }
    fn y(self) -> T { self[1] }
    fn z(self) -> T { self[2] }
    fn w(self) -> T { self[3] }
}

impl<T> Array<T> for [T, ..16] {
    fn from_fn(f: |uint| -> T) -> [T, ..16] {
        [
            f(0), f(1), f(2), f(3),
            f(4), f(5), f(6), f(7),
            f(8), f(9), f(10),f(11),
            f(12),f(13),f(14),f(15)
        ]
    }
}
