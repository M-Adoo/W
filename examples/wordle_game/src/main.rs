#![feature(test, return_position_impl_trait_in_trait)]

use ribir::prelude::*;
use ribir_dev_helper::*;

use crate::ui::wordle_game;
mod ui;
mod wordle;

example_framework!(wordle_game, wnd_size = Size::new(700., 620.));
