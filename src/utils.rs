/// Converts a hex color to a tuple of (r, g, b)
pub fn hex_to_rgb(hex: &str) -> Result<(u8, u8, u8), Box<dyn std::error::Error>> {
    let hex = hex.trim_start_matches("#");
    let r = u8::from_str_radix(&hex[0..2], 16)?;
    let g = u8::from_str_radix(&hex[2..4], 16)?;
    let b = u8::from_str_radix(&hex[4..6], 16)?;

    Ok((r, g, b))
}

// Converts a position on the Voyager to the corresponding key index
pub fn pos_to_voyager(x: u16, y: u16) -> usize {
    // 0,  5   is left 1st row
    // 6,  11  is left 2st row
    // 12, 17  is left 3st row
    // 18, 23  is left 4st row

    // 24, 25 is left thumb keys
    // 50, 51 is right thumb keys

    // 26, 31  is right 1st row
    // 32, 37  is right 2st row
    // 38, 43  is right 3st row
    // 44, 49  is right 4st row

    #[rustfmt::skip]
    let voyager_layout: [[usize; 12]; 5] = [
        [ 0,  1,  2,  3,  4,  5,        26, 27, 28, 29, 30, 31],
        [ 6,  7,  8,  9, 10, 11,        32, 33, 34, 35, 36, 37],
        [12, 13, 14, 15, 16, 17,        38, 39, 40, 41, 42, 43],
        [18, 19, 20, 21, 22, 23,        44, 45, 46, 47, 48, 49],

        [60, 60, 60, 60, 24, 25,        50, 51, 60, 60, 60, 60]
    ];
    voyager_layout[y as usize][x as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_hex_to_rgb() {
        let hex = "#ff0000";
        let (r, g, b) = hex_to_rgb(hex).unwrap();
        assert_eq!(r, 255);
        assert_eq!(g, 0);
        assert_eq!(b, 0);
    }

    #[test]
    fn convert_hex_to_rgb_without_pound() {
        let hex = "3edece";
        let (r, g, b) = hex_to_rgb(hex).unwrap();
        assert_eq!(r, 62);
        assert_eq!(g, 222);
        assert_eq!(b, 206);
    }
}
