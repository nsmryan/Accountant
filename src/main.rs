use std::fs::File;

use csv;


type Count = u32;


#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Pattern {
    /// Number of dimensions in which the counting pattern takes place.
    dims: u32,

    /// The rollover value for the pattern- the maximum value a count can take before
    /// rolling back to 0
    rollover: Count,

    /// The step when crossing between dimensions. This is a count per dimension,
    /// but may be 0 if the count resets.
    steps: Vec<Count>,

    /// The number of elements in each dimension of the pattern. This is a size
    /// per dimension, but the last dimension may be ommitted to indicate that
    /// there may be any number of repetitions of that dimension.
    sizes: Vec<Count>,
}

impl Pattern {
    pub fn next_expected(&self, count: Count) -> Count {
        // NOTE incomplete
        count + 1
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum PatternType {
    Square,
    Jagged,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct PatternState {
    pub pattern: Pattern,
    pub state: CountState,
    pub pattern_type: PatternType,
    pub resync_expected: usize,
    pub resync_actual: usize,
    pub unexpected_counts: usize,
}

impl PatternState {
    pub fn make_pattern_state(pattern: Pattern, pattern_type: PatternType, start: Option<Count>) -> PatternState {
        let counting_state = match start {
            Some(starting_count) => CountState::Counting(starting_count),
            None => CountState::Uninitialized,
        };

        PatternState {
            pattern: pattern,
            state: counting_state,
            pattern_type: pattern_type,
            resync_expected: 0,
            resync_actual: 0,
            unexpected_counts: 0,
        }
    }
}

impl PatternState {
    pub fn count(&mut self, count: Count) {

        match self.state {
            CountState::Uninitialized => {
                self.state = CountState::Counting(count);
            },

            CountState::Counting(prev_expected) => {
                let next_expected = self.pattern.next_expected(count);

                if prev_expected == count {
                    self.state = CountState::Counting(next_expected);
                } else {
                    let err_next_expected = match self.pattern_type {
                        PatternType::Square => next_expected,
                        PatternType::Jagged => prev_expected,
                    };
                    self.state = CountState::Unexpected(err_next_expected, count);
                }
            }

            CountState::Unexpected(expected, actual) => {
                let next_expected = self.pattern.next_expected(count);
                let actual_next_expected = self.pattern.next_expected(actual);

                // we are back in sync with the expected pattern
                if expected == count {
                    self.resync_expected += 1;
                    self.state = CountState::Counting(next_expected);

                // we are back in sync with the actual pattern
                } else if actual_next_expected == count {
                    self.resync_actual += 1;
                    self.state = CountState::Counting(actual_next_expected);

                // we are still not in sync
                } else {
                    self.unexpected_counts += 1;
                    self.state = CountState::Unexpected(next_expected, count);
                }
            },
        }
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum CountState {
    /// No count seen
    Uninitialized,            

    /// Next expected count
    Counting(Count),          

    /// Expected, Actual
    Unexpected(Count, Count), 
}

impl CountState {
    pub fn process(&self, count: Count) -> CountState {
        match self {
            CountState::Uninitialized => {
                CountState::Counting(count)
            },

            CountState::Counting(last_count) => {
                if last_count + 1 == count {
                    CountState::Counting(count)
                } else {
                    CountState::Unexpected(last_count + 1, count)
                }
            }

            CountState::Unexpected(expected, actual) => {
                if *expected == count || *actual + 1 == count {
                    CountState::Counting(count)
                } else {
                    // NOTE should be configurable whether the new expected is
                    // count + 1, or remains the same as the previous expected value
                    CountState::Unexpected(*expected, count)
                }
            },
        }
    }
}


fn main() {
    let col_name = "header";
    let mut col_index = 0;

    let mut rollover = 10;

    let mut counting_state = CountState::Uninitialized;

    let pattern = Pattern {
        dims: 1,
        rollover: 4,
        steps: vec!(0),
        sizes: vec!(13),
    };

    let pattern_state =
        PatternState::make_pattern_state(pattern, PatternType::Square, None);

    let mut reader = csv::Reader::from_reader(File::open("test.csv").unwrap());

    if !reader.has_headers() {
        panic!("Expected a csv file with a header!");
    }

    for header in reader.headers().unwrap() {
        if col_name == header {
            println!("Found {} at col {}", col_name, col_index);
        }
        else {
            println!("{:?}", header);
            col_index += 1;
        }
    }


    for result in reader.records() {
        let record = result.unwrap();

        let val = record.get(col_index).unwrap().parse::<Count>().unwrap();

        counting_state = counting_state.process(val);

        println!("{:?}", counting_state);
    }
}

