use embassy_nrf::{
    Peri,
    gpio::{Input, Level, Output, OutputDrive, Pull},
    peripherals::{
        NVMC, P0_04, PPI_CH17, PPI_CH18, PPI_CH19, PPI_CH20, PPI_CH21, PPI_CH22, PPI_CH23,
        PPI_CH24, PPI_CH25, PPI_CH26, PPI_CH27, PPI_CH28, PPI_CH29, PPI_CH30, PPI_CH31, RNG, RTC0,
        SAADC, TEMP, TIMER0,
    },
};

use crate::matrix::Matrix;

pub struct BlePeri {
    pub ppi_ch17: Peri<'static, PPI_CH17>,
    pub ppi_ch18: Peri<'static, PPI_CH18>,
    pub ppi_ch19: Peri<'static, PPI_CH19>,
    pub ppi_ch20: Peri<'static, PPI_CH20>,
    pub ppi_ch21: Peri<'static, PPI_CH21>,
    pub ppi_ch22: Peri<'static, PPI_CH22>,
    pub ppi_ch23: Peri<'static, PPI_CH23>,
    pub ppi_ch24: Peri<'static, PPI_CH24>,
    pub ppi_ch25: Peri<'static, PPI_CH25>,
    pub ppi_ch26: Peri<'static, PPI_CH26>,
    pub ppi_ch27: Peri<'static, PPI_CH27>,
    pub ppi_ch28: Peri<'static, PPI_CH28>,
    pub ppi_ch29: Peri<'static, PPI_CH29>,
    pub ppi_ch30: Peri<'static, PPI_CH30>,
    pub ppi_ch31: Peri<'static, PPI_CH31>,
    pub rtc0: Peri<'static, RTC0>,
    pub timer0: Peri<'static, TIMER0>,
    pub temp: Peri<'static, TEMP>,
    pub nvmc: Peri<'static, NVMC>,
    pub rng: Peri<'static, RNG>,
    pub p_04: Peri<'static, P0_04>,
    pub saadc: Peri<'static, SAADC>,
}

pub struct AppPeri<'a> {
    pub ble_peri: BlePeri,
    pub matrix_peri: Matrix<'a>,
}

impl<'a> AppPeri<'a> {
    pub fn new() -> Self {
        // init peripherals
        let p = embassy_nrf::init(Default::default());

        // init ble peripherals
        let ble_peri = BlePeri {
            ppi_ch17: p.PPI_CH17,
            ppi_ch18: p.PPI_CH18,
            ppi_ch19: p.PPI_CH19,
            ppi_ch20: p.PPI_CH20,
            ppi_ch21: p.PPI_CH21,
            ppi_ch22: p.PPI_CH22,
            ppi_ch23: p.PPI_CH23,
            ppi_ch24: p.PPI_CH24,
            ppi_ch25: p.PPI_CH25,
            ppi_ch26: p.PPI_CH26,
            ppi_ch27: p.PPI_CH27,
            ppi_ch28: p.PPI_CH28,
            ppi_ch29: p.PPI_CH29,
            ppi_ch30: p.PPI_CH30,
            ppi_ch31: p.PPI_CH31,
            rtc0: p.RTC0,
            timer0: p.TIMER0,
            temp: p.TEMP,
            nvmc: p.NVMC,
            rng: p.RNG,
            p_04: p.P0_04,
            saadc: p.SAADC,
        };

        // init rows
        let rows = [
            Output::new(p.P0_17, Level::Low, OutputDrive::Standard),
            Output::new(p.P0_20, Level::Low, OutputDrive::Standard),
            Output::new(p.P0_22, Level::Low, OutputDrive::Standard),
            Output::new(p.P0_24, Level::Low, OutputDrive::Standard),
        ];

        // init cols
        let cols = [
            Input::new(p.P0_31, Pull::Down),
            Input::new(p.P0_29, Pull::Down),
            Input::new(p.P0_02, Pull::Down),
            Input::new(p.P1_15, Pull::Down),
            Input::new(p.P1_13, Pull::Down),
        ];

        // init matrix
        let matrix_peri = Matrix::init(rows, cols);

        Self {
            ble_peri,
            matrix_peri,
        }
    }
}
