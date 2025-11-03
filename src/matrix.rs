use embassy_nrf::gpio::{Input, Output};

use crate::config::{COLS, ROWS};

pub async fn scan_matrix<'a>(rows: [Output<'a>; ROWS], cols: [Input<'a>; COLS]) {}
