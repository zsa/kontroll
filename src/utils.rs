pub fn hex_to_rgb(hex: &str) -> Result<(u8, u8, u8), Box<dyn std::error::Error>> {
    let hex = hex.trim_start_matches("#");
    let r = u8::from_str_radix(&hex[0..2], 16)?;
    let g = u8::from_str_radix(&hex[2..4], 16)?;
    let b = u8::from_str_radix(&hex[4..6], 16)?;

    Ok((r, g, b))
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
