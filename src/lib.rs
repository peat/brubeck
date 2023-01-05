//! # Brubeck: A RISC-V REPL
//!
//! Provides a "read, evaluate, print, loop" interactive environment for RISC-V assembly language.
//! What could be more fun?!
//!
//! ## Under Construction
//!
//! Don't expect this to work out of the box. I'm working the kinks out of the RV32I implementation
//! to start; interactivity coming soon!
//!
//! ## Name & Tribute
//!
//! From Wikipedia: _"[Dave Brubeck](https://en.wikipedia.org/wiki/Dave_Brubeck) was an American
//! jazz pianist and composer. Often regarded as a foremost exponent of cool jazz, Brubeck's work
//! is characterized by unusual time signatures and superimposing contrasting rhythms, meters, and
//! tonalities. [His song] "Take Five" became the highest-selling jazz single of all time."_

mod immediate;
pub mod rv32_i;

pub use immediate::Immediate;
