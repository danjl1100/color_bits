pub use bits::U8Iterator;
pub use color::Color;
pub use color::ColorIterator;

/// Bit-wise manipuation for iterating primitive types.
pub mod bits {
    /// Iterates `u8` type from MSB to LSB, outputting one `bool` for each bit.
    pub struct U8Iterator {
        value: u8,
        remaining: usize,
    }
    impl U8Iterator {
        /// Constructs an "empty" iterator, which only returns `None`.
        /// To be used in conjunction with [`reset_to()`].
        ///
        /// [`reset_to()`]: struct.U8Iterator.html#method.reset_to
        ///
        /// ```
        /// use color_bits::U8Iterator;
        /// let mut iter = U8Iterator::empty();
        /// for _ in 0..=100 {
        ///     assert_eq!(iter.next(), None);
        /// }
        /// ```
        pub fn empty() -> U8Iterator {
            U8Iterator {
                value: 0,
                remaining: 0,
            }
        }
        /// Resets the iterator to the specified `value`, with 8 bits remaining to be output.
        ///
        /// ```
        /// use color_bits::U8Iterator;
        /// let mut iter = U8Iterator::from(0b0101_0101);
        /// assert_eq!(iter.next(), Some(false));
        /// assert_eq!(iter.next(), Some(true));
        /// assert_eq!(iter.next(), Some(false));
        /// assert_eq!(iter.next(), Some(true));
        /// // allows reset mid-iteration to accept new value
        /// iter.reset_to(0b1100_0000);
        /// assert_eq!(iter.next(), Some(true));
        /// assert_eq!(iter.next(), Some(true));
        /// assert_eq!(iter.next(), Some(false));
        /// // reset again
        /// iter.reset_to(255);
        /// // deplete iterator
        /// for _ in 0..=7 {
        ///     assert_eq!(iter.next(), Some(true));
        /// }
        /// assert_eq!(iter.next(), None);
        /// ```
        pub fn reset_to(&mut self, value: u8) {
            self.value = value;
            self.remaining = 8;
        }
    }
    impl From<u8> for U8Iterator {
        fn from(value: u8) -> U8Iterator {
            let mut iter = U8Iterator::empty();
            iter.reset_to(value);
            iter
        }
    }
    impl Iterator for U8Iterator
    {
        type Item = bool;
        /// Returns bits of the `u8` value from MSB to LSB, outputting one `bool` for each bit.
        /// ```
        /// use color_bits::U8Iterator;
        /// let iter = U8Iterator::from(255_u8);
        /// let bits: Vec<bool> = iter.collect();
        /// assert_eq!(bits, [true; 8]);
        /// //
        /// let iter = U8Iterator::from(0_u8);
        /// let bits = iter.collect::<Vec<bool>>();
        /// assert_eq!(bits, [false; 8]);
        /// //
        /// let iter = U8Iterator::from(0b1010_1010);
        /// let bits = iter.collect::<Vec<bool>>();
        /// assert_eq!(bits, [true, false, true, false, true, false, true, false]);
        /// ```
        fn next(&mut self) -> Option<bool> {
            if self.remaining == 0 {
                None
            } else {
                // calc MSB
                const MSB_MASK: u8 = 0b1000_0000;
                let bit = self.value & MSB_MASK;
                let bit = bit > 0;
                // advance to next bit
                self.remaining -= 1;
                self.value <<= 1;
                //
                Some(bit)
            }
        }
        /// ```
        /// use color_bits::U8Iterator;
        /// let mut iter = U8Iterator::from(0_u8);
        /// for i in (1..=8).rev() {
        ///     assert_eq!(iter.size_hint(), (i, Some(i)));
        ///     assert_eq!(iter.next(), Some(false));
        /// }
        /// assert_eq!(iter.size_hint(), (0, Some(0)));
        /// assert_eq!(iter.next(), None);
        /// ```
        fn size_hint(&self) -> (usize, Option<usize>) {
            (self.remaining, Some(self.remaining))
        }
    }
}

/// Color representations and the associated iterators.
pub mod color {
    use super::U8Iterator;
    use core::marker::PhantomData;
    use Component::*;
    /// 24-bit representation of red, green, and blue color components.
    pub struct Color {
        pub green: u8,
        pub red: u8,
        pub blue: u8,
    }
    impl Color {
        /// Constructs a new color for the given `red`, `green`, and `blue` components.
        pub fn new(red: u8, green: u8, blue: u8) -> Color {
            Color { red, green, blue }
        }
        /// ```
        /// use color_bits::Color;
        /// let black = Color::new(0, 0, 0);
        /// let iter = black.into_iter_gbr();
        /// let bits = iter.collect::<Vec<bool>>();
        /// assert_eq!(bits, [false; 24]);
        /// //
        /// let white = Color::new(255, 255, 255);
        /// let bits = white.into_iter_gbr().collect::<Vec<bool>>();
        /// assert_eq!(bits, [true; 24]);
        /// //
        /// let pink = Color::new(255, 0b1010_1010, 0b1110_0001);
        /// let mut iter = pink.into_iter_gbr();
        /// for _ in 0..=3 { //green
        ///     assert_eq!(iter.next(), Some(true));
        ///     assert_eq!(iter.next(), Some(false));
        /// }
        /// for _ in 0..=7 { //red
        ///     assert_eq!(iter.next(), Some(true));
        /// }
        /// for _ in 0..=2 { //blue
        ///     assert_eq!(iter.next(), Some(true));
        /// }
        /// for _ in 0..=3 {
        ///     assert_eq!(iter.next(), Some(false));
        /// }
        /// assert_eq!(iter.next(), Some(true));
        /// assert_eq!(iter.next(), None);
        /// ```
        pub fn into_iter_gbr(self) -> ColorIterator<OrderGBR> {
            self.into_iter()
        }
        pub fn into_iter<Order: ColorOrder>(self) -> ColorIterator<Order> {
            ColorIterator::new(self)
        }
    }
    /// Iterates color values using specified [`ColorOrder`] type implementation.
    ///
    /// [`ColorOrder`]: trait.ColorOrder.html
    pub struct ColorIterator<Order: ColorOrder> {
        color: Color,
        iter: U8Iterator,
        component: Option<Component>,
        phantom: PhantomData<Order>,
    }
    impl<Order: ColorOrder> ColorIterator<Order> {
        fn new(color: Color) -> ColorIterator<Order> {
            let component = Order::first();
            let iter = U8Iterator::from(component.select_from(&color));
            ColorIterator {
                color,
                iter,
                component: Some(component),
                phantom: PhantomData,
            }
        }
    }
    impl<Order: ColorOrder> Iterator for ColorIterator<Order> {
        type Item = bool;
        fn next(&mut self) -> Option<bool> {
            if let Some(value) = self.iter.next() {
                Some(value)
            } else {
                // advance self.component
                if let Some(component) = &self.component {
                    self.component = Order::next(&component);
                    if let Some(component) = &self.component {
                        // Iterate next color value
                        self.iter.reset_to(component.select_from(&self.color));
                        self.iter.next()
                    } else {
                        // Next component is Done
                        None
                    }
                } else {
                    // Currently Done
                    None
                }
            }
        }
    }
    /// Definition of red, green, and blue components.
    /// For use in [`ColorOrder`] implementations.
    ///
    /// [`ColorOrder`]: trait.ColorOrder.html
    pub enum Component {
        Red,
        Green,
        Blue,
    }
    impl Component {
        fn select_from(&self, color: &Color) -> u8 {
            match self {
                Green => color.green,
                Red => color.red,
                Blue => color.blue,
            }
        }
    }
    /// Specifies iteration order of color [`Component`]s.
    ///
    /// [`Component`]: enum.Component.html
    pub trait ColorOrder {
        /// Returns the first color component
        fn first() -> Component;
        /// Returns the next color component
        fn next(current: &Component) -> Option<Component>;
    }
    /// Implements `Green`, `Red`, `Blue` ordering.
    pub struct OrderGBR {}
    impl ColorOrder for OrderGBR {
        fn first() -> Component {
            Green
        }
        fn next(component: &Component) -> Option<Component> {
            match component {
                Green => Some(Red),
                Red => Some(Blue),
                Blue => None,
            }
        }
    }
}
