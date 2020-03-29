use vob::{Vob};
use std::io::Write;

type Num = usize;
const BASE: Num = 10;
const N_DIGITS: usize = 7;

fn main() -> Result<(),std::io::Error> {
    let max = BASE.checked_pow(N_DIGITS as u32).expect("Num type too small") - 1;
    assert!(max <= usize::max_value() as Num, "usize cannot address max");

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    let mut free = Vob::from_elem((max+1) as usize, false);

    let mut current: Num = 0;
    let mut count: usize = 0;
    while let Some(next_free) = free.iter_unset_bits((current+1)..).next() {
        count += 1;
        current = next_free as Num;
        writeln!(&mut out, "{:0digits$}", current, digits = N_DIGITS-1)?;

        let mut exp: Num = 1;
        for _ in 0..N_DIGITS {
            // digit_post = 2, exp = 100, current = 3456, current_digit = 4
            let current_digit: Num = (current / exp % exp) % 10;
            // current_level = 3056
            let current_level: Num = current - current_digit * exp;
            for change_to_digit in 0..BASE {
                let with_changed_digit: Num = current_level + change_to_digit*exp;
                free.set(with_changed_digit as usize, true);
            }

            exp *= BASE;
        }
    }

    writeln!(&mut out, "total count: {}", count)?;

    Ok(())
}
