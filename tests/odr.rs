macro_rules! assert_odr_from_hertz {
    ($n:expr => Some($hz:ident)) => {{
        assert_eq!(Odr::from_hertz($n), Some(Odr::$hz));
    }};
    ($n:expr => None) => {{
        assert_eq!(Odr::from_hertz($n), None);
    }};
}

#[test]
fn acc_odr_from_hz() {
    use lsm303agr::AccelOutputDataRate as Odr;

    assert_odr_from_hertz!(0 => None);
    assert_odr_from_hertz!(1 => Some(Hz1));
    assert_odr_from_hertz!(10 => Some(Hz10));
    assert_odr_from_hertz!(20 => None);
    assert_odr_from_hertz!(25 => Some(Hz25));
    assert_odr_from_hertz!(50 => Some(Hz50));
    assert_odr_from_hertz!(100 => Some(Hz100));
    assert_odr_from_hertz!(200 => Some(Hz200));
    assert_odr_from_hertz!(400 => Some(Hz400));
    assert_odr_from_hertz!(3333 => None);
    assert_odr_from_hertz!(1344 => Some(Khz1_344));
    assert_odr_from_hertz!(1620 => Some(Khz1_620LowPower));
    assert_odr_from_hertz!(5376 => Some(Khz5_376LowPower));
}

#[test]
fn mag_odr_from_hz() {
    use lsm303agr::MagOutputDataRate as Odr;

    assert_odr_from_hertz!(0 => None);
    assert_odr_from_hertz!(1 => None);
    assert_odr_from_hertz!(10 => Some(Hz10));
    assert_odr_from_hertz!(20 => Some(Hz20));
    assert_odr_from_hertz!(33 => None);
    assert_odr_from_hertz!(50 => Some(Hz50));
    assert_odr_from_hertz!(100 => Some(Hz100));
    assert_odr_from_hertz!(333 => None);
}
