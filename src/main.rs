use vob::{Vob};
use std::io::Write;

type Num = usize;
const BASE: Num = 10;
const N_DIGITS: usize = 9;

fn main() {
    let start_time = std::time::SystemTime::now();

    let max = BASE.checked_pow(N_DIGITS as u32).expect("Num type too small") - 1;
    assert!(max <= usize::max_value() as Num, "usize cannot address max");

    let stdout = std::io::stdout();
    let mut out = stdout.lock();

    // A bit set to keep track of all numbers which are too similar to already
    // chosen numbers (e.g. only differ in one digit).
    // We start out with no numbers being too close.
    let mut too_similar = Vob::from_elem((max+1) as usize, false);

    let mut current: Num = 0;
    let mut count: usize = 0;

    // The number of leading zeros to print before the number.
    let mut leading_zeroes = N_DIGITS;
    // The next number at which we need to remove one zero.
    let mut next_zero = 0;

    // Get the next free number.
    while let Some(next_free) = too_similar.iter_unset_bits(current..).next() {
        count += 1;
        current = next_free as Num;

        while next_zero <= current {
            if next_zero == 0 {
                next_zero = 1;
            }
            leading_zeroes -= 1;
            next_zero *= BASE;
        }

        for _ in 0..leading_zeroes {
            out.write_all(b"0").expect("write failed");
        }

        itoa::write(&mut out, current).expect("itoa failed");
        out.write_all(b"\n").expect("write failed");


        // Mark all numbers which differ only by one digit as unsuitable.

        // exp = 1 for the first digit, 10 for the second, 100 for the third...
        let mut exp: Num = 1;
        // Iterate over all digit positions.
        for _ in 0..N_DIGITS {
            // The current digit at the selected position.
            // 
            // Example: current = 3456, digit_post = 2, exp = 100
            //          => current_digit = 4
            let current_digit: Num = (current / exp % exp) % 10;
            // The curent number with the selected digit set to zero.
            //
            // Example: 
            // 
            // current = 3456, digit_post = 2, exp = 100
            // => current_level = 3056
            let current_level: Num = current - current_digit * exp;
            // Now change the digit in the selected position to all possible
            // values. Note that this includes the original value since this
            // also has to be excluded.
            for change_to_digit in 0..BASE {
                // The current number with the selected digit changed to
                // change_to_digit.
                //
                // Example: 
                //
                // exp = 100, current_level = 3056, change_to_digit = 9
                // => with_changed_digit = 3956
                let with_changed_digit: Num = current_level + change_to_digit*exp;
                too_similar.set(with_changed_digit as usize, true);
            }

            exp *= BASE;
        }
    }

    eprintln!("finished, total count: {}", count);
    eprintln!("Took {} ms", start_time.elapsed().expect("elapsed failed").as_millis());
}
