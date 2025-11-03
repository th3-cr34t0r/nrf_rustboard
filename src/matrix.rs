use core::pin::pin;

use embassy_futures::select::{Either, select, select_slice};
use embassy_nrf::gpio::{Input, Output};
use heapless::Vec;
use nrf_rustboard::{delay_ms, delay_us};

use crate::config::{ASYNC_ROW_WAIT, COLS, REGISTERED_KEYS_BUFFER, ROWS};

#[derive(Default, Clone, Copy, PartialEq)]
pub struct KeyPos {
    row: u8,
    col: u8,
}

impl KeyPos {
    pub fn default() -> Self {
        Self { row: 255, col: 255 }
    }
}

pub struct Matrix<'a> {
    rows: [Output<'a>; ROWS],
    cols: [Input<'a>; COLS],
    registered_keys: [KeyPos; REGISTERED_KEYS_BUFFER],
}

impl<'a> Matrix<'a> {
    pub fn init(rows: [Output<'a>; ROWS], cols: [Input<'a>; COLS]) -> Self {
        let registered_keys = [KeyPos::default(); REGISTERED_KEYS_BUFFER];

        Self {
            rows,
            cols,
            registered_keys,
        }
    }

    // main function for scanning and registering keys
    pub async fn async_scan(&mut self) {
        for (row_count, row) in self.rows.iter_mut().enumerate() {
            row.set_high();
            // delay so port propagates
            delay_us(1).await;

            // set cols wait for high
            {
                let mut futures: Vec<_, COLS> = self
                    .cols
                    .iter_mut()
                    .map(|col| col.wait_for_high())
                    .collect();

                match select(
                    select_slice(pin!(futures.as_mut_slice())),
                    delay_ms(ASYNC_ROW_WAIT),
                )
                .await
                {
                    Either::First(_) => {
                        // key has been pressed
                    }
                    Either::Second(()) => {
                        // time is up, continue with the next row
                        row.set_low();
                        continue;
                    }
                }
            }

            // get the pressed keys
            for (col_count, col) in self.cols.iter().enumerate() {
                if col.is_high() {
                    // store the registere key in an empty element
                    if let Some(index) = self
                        .registered_keys
                        .iter()
                        .position(|element| *element == KeyPos::default())
                    {
                        self.registered_keys[index] = KeyPos {
                            row: row_count as u8,
                            col: col_count as u8,
                        };
                    }
                }
            }

            row.set_low();
        }

        // TODO: Store the registered local keys in a globally shared variable
    }
}

pub async fn scan_matrix<'a>(mut matrix_peri: Matrix<'a>) {
    loop {
        // run the matrix scan
        matrix_peri.async_scan().await;
    }
}
