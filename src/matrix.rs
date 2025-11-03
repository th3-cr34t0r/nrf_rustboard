use core::pin::pin;

use embassy_futures::select::{Either, select, select_slice};
use embassy_nrf::gpio::{Input, Output};
use heapless::Vec;
use nrf_rustboard::delay_us;

use crate::config::{ASYNC_ROW_WAIT, COLS, REGISTERED_KEYS_BUFFER, ROWS};

pub struct KeyPos {
    row: u8,
    col: u8,
}

impl KeyPos {
    pub fn default() -> Self {
        Self { row: 255, col: 255 }
    }
}

pub struct Matrix {
    rows: [Output<'a>; ROWS],
    cols: [Input<'a>; COLS],
    registered_keys: [KeyPos; REGISTERED_KEYS_BUFFER],
}

impl Matrix {
    pub fn init() -> Self {
        // init rows
        let rows = [
            Output::new(p.P0_17, Level::Low, OutputDrive::Standard),
            Output::new(p.P0_20, Level::Low, OutputDrive::Standard),
            Output::new(p.P0_22, Level::Low, OutputDrive::Standard),
            Output::new(p.P0_24, Level::Low, OutputDrive::Standard),
        ];

        // init cols
        let cols = [
            Input::new(p.P0_31, Pull::Up),
            Input::new(p.P0_29, Pull::Up),
            Input::new(p.P0_02, Pull::Up),
            Input::new(p.P1_15, Pull::Up),
            Input::new(p.P1_13, Pull::Up),
        ];

        let registered_keys = [KeyPos::default(); REGISTERED_KEYS_BUFFER];

        Self {
            rows,
            cols,
            registered_keys,
        }
    }

    // main function for scanning and registering keys
    pub async fn async_scan(&mut self) {
        for (row_count, row) in self.rows.iter().enumerate() {
            row.set_high();
            // delay so port propagates
            delay_us(10).await;

            let mut futures: Vec<_, COLS> = self
                .cols
                .iter_mut()
                .map(|col| col.wait_for_high())
                .collect();

            match select(
                select_slice(pin!(futures.as_mut_slice())),
                delay_us(ASYNC_ROW_WAIT),
            )
            .await
            {
                Either::First((Ok(_), _)) => {
                    // do something in case button is pressed
                    for (col_count, col) in self.cols.iter().enumerate() {
                        if col.is_high() {
                            // store the registere key in an empty element
                            if let Some(index) = self
                                .registered_keys
                                .iter()
                                .position(|element| element == KeyPos::default())
                            {
                                self.registered_keys[index] = KeyPos {
                                    row: row_count,
                                    col: col_count,
                                };
                            }
                        }
                    }
                }
                Either::First((Err(_), _)) => {}
                Either::Second(()) => {} // time is up, continue with the next row
            }

            row.set_low();
        }

        // TODO: Store the registered local keys in a globally shared variable
    }
}

pub async fn scan_matrix<'a>(matrix_peri: Matrix) {
    loop {
        // run the matrix scan
        matrix_peri.async_scan().await;
    }
}
