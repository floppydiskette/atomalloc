#![no_std]

#[cfg(feature = "no_alloc")]
pub mod noalloc;
#[cfg(feature = "with_alloc")]
pub mod withalloc;

#[cfg(test)]
mod tests;