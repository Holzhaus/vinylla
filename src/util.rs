// Copyright (c) 2025 Jan Holthuis <holthuis.jan@gmail.com> et al.
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v. 2.0. If a copy
// of the MPL was not distributed with this file, You can obtain one at
// http://mozilla.org/MPL/2.0/.
//
// SPDX-License-Identifier: MPL-2.0

//! Helper Utilities

/// Discrete-time implementation of a simple RC low-pass filter to calculate the exponential
/// weighted moving average.
#[derive(Debug, Clone, PartialEq)]
pub struct ExponentialWeightedMovingAverage {
    /// The smoothed last output.
    pub last_output: i32,

    /// The smoothing factor (commonly named α in literature). Needs to be in range 0.0 − 1.0.
    /// (inclusive).
    pub smoothing_factor: f64,
}

impl ExponentialWeightedMovingAverage {
    pub fn new(time_constant: f64, sample_rate_hz: f64) -> Self {
        let last_output = 0;
        let smoothing_factor = Self::calculate_smoothing_factor(time_constant, sample_rate_hz);
        ExponentialWeightedMovingAverage {
            last_output,
            smoothing_factor,
        }
    }

    /// Calculate the smoothing factor.
    ///
    /// Using the time constant RC and and the sample rate f_s, this calculates the
    /// smoothing factor α:
    ///
    /// Δ_T = 1/f_s
    /// α = Δ_T / (RC + Δ_T)
    ///
    /// where Δ_T is the sampling period.
    fn calculate_smoothing_factor(time_constant: f64, sample_rate_hz: f64) -> f64 {
        let sampling_period_secs = 1f64 / sample_rate_hz;
        sampling_period_secs / (time_constant + sampling_period_secs)
    }

    /// Calculate the difference between the current input and last output value.
    pub fn difference_to(&self, input: i32) -> i32 {
        input - self.last_output
    }

    /// Calculate the next smoothed value.
    ///
    /// This calculates the next smoothed value yᵢ using the previous smoothed value yᵢ₋₁, the
    /// current unsmoothed value xᵢ and the smoothing factor α:
    ///
    /// yᵢ = α ⋅ xᵢ + (1 − α) ⋅ yᵢ₋₁
    ///
    /// To avoid unnecessary floating point calculations, the above equation can be written as:
    ///
    /// yᵢ = α ⋅ xᵢ + (1 − α) ⋅ yᵢ₋₁
    /// = α ⋅ xᵢ + yᵢ₋₁ − α ⋅ yᵢ₋₁
    /// = yᵢ₋₁ + α ⋅ xᵢ − α ⋅ yᵢ₋₁
    /// = yᵢ₋₁ + alpha ⋅ (xᵢ − yᵢ₋₁)
    pub fn smoothen(&self, input: i32) -> i32 {
        self.last_output + (self.smoothing_factor * self.difference_to(input) as f64) as i32
    }

    /// Calculate the next smoothed value and store it.
    pub fn process(&mut self, input: i32) -> i32 {
        self.last_output = self.smoothen(input);
        self.last_output
    }
}
