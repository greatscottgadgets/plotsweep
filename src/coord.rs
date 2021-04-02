use plotters::coord::ranged1d::{
    AsRangedCoord, DefaultFormatting, KeyPointHint, Ranged,
};
use std::ops::Range;

/// This axis decorator will reverse the direction of the axis.
#[derive(Clone)]
pub struct ReversedAxis<R: Ranged>(R);

/// The trait for the types that can be converted into a reversed axis
pub trait IntoReversedAxis: AsRangedCoord {
    /// Make the reversed axis
    ///
    /// - **returns**: The converted range specification
    fn reversed_axis(
        self,
    ) -> ReversedAxis<Self::CoordDescType> {
        ReversedAxis(self.into())
    }
}

impl<R: AsRangedCoord> IntoReversedAxis for R {}

impl<R: Ranged> Ranged for ReversedAxis<R>
where
    R::ValueType: Clone,
{
    type FormatOption = DefaultFormatting;
    type ValueType = R::ValueType;

    fn map(&self, value: &Self::ValueType, limit: (i32, i32)) -> i32 {
        self.axis_pixel_range(limit).end - self.0.map(value, limit)
    }

    fn key_points<Hint: KeyPointHint>(&self, hint: Hint) -> Vec<Self::ValueType> {
        self.0.key_points(hint)
    }

    fn range(&self) -> Range<Self::ValueType> {
        self.0.range()
    }

    fn axis_pixel_range(&self, limit: (i32, i32)) -> Range<i32> {
        self.0.axis_pixel_range(limit)
    }
}