type XP = i32;
type Level = i32;

fn calc(xp: XP) -> (XP, Level) {
    // First, 100 then 10%
    
    let mut n : (XP, Level) = (100, 0);
    
    while n.0 <= xp {
        n = (((n.0 as f32) * 1.1) as i32, n.1 + 1);
    }

    n
}

/// Takes the current level of experience and returns the next level which is required
pub fn get_amount_of_xp_required(xp: XP) -> XP {
    let (n, _) = calc(xp);
    n
}

/// Takes the current level of experience and returns the current level
pub fn get_lvl_by_xp(xp: XP) -> Level {
    let (_, n) = calc(xp);
    n
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_amount_lvl_1() {
        let xp = 0; 

        let xp_required = get_amount_of_xp_required(xp);

        assert_eq!(xp_required, 100);
    }

    #[test]
    fn test_get_amount_lvl_2() {
        let xp = 100; 

        let xp_required = get_amount_of_xp_required(xp);

        assert_eq!(xp_required, 110);
    }

    #[test]
    fn test_get_amount_lvl_10() {
        let xp = 235; 

        let xp_required = get_amount_of_xp_required(xp);

        assert_eq!(xp_required, 256);
    }

    #[test]
    fn test_get_lvl_0() {
        let xp = 0; 

        let lvl = get_lvl_by_xp(xp);

        assert_eq!(lvl, 0);
    }

    #[test]
    fn test_get_lvl_1() {
        let xp = 100; 

        let lvl = get_lvl_by_xp(xp);

        assert_eq!(lvl, 1);
    }

    #[test]
    fn test_get_lvl_10() {
        let xp = 235; 

        let lvl = get_lvl_by_xp(xp);

        assert_eq!(lvl, 10);
    }
}

