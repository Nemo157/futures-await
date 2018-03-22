use __rt::pin_api::boxed::PinBox;

pub auto trait NotBox {
}

impl<T> !NotBox for Box<T> {
}

impl<T> !NotBox for PinBox<T> {
}

pub trait MaybeBox<T> {
    type Boxed;

    fn maybe_into_box(value: T) -> Self::Boxed;
}

impl<T> MaybeBox<T> for Box<T> {
    type Boxed = Box<T>;

    fn maybe_into_box(value: T) -> Self::Boxed { Box::new(value) }
}

impl<T> MaybeBox<T> for PinBox<T> {
    type Boxed = PinBox<T>;

    fn maybe_into_box(value: T) -> Self::Boxed { PinBox::new(value) }
}

impl<T: NotBox> MaybeBox<T> for T {
    type Boxed = T;

    fn maybe_into_box(value: T) -> Self::Boxed { value }
}
