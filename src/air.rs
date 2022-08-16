// Copyright Vesnica
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use winter_air::{
    Air, AirContext, Assertion, EvaluationFrame, ProofOptions, TraceInfo,
    TransitionConstraintDegree,
};
use winter_math::FieldElement;
use winter_prover::TraceTable;
use winter_utils::{ByteWriter, Serializable};

use base64::{decode, encode};
use clap::Args;
use serde::{Deserialize, Serialize};

pub type BaseElement = winter_math::fields::f128::BaseElement;

#[derive(Args, Debug)]
#[clap(next_help_heading = "INPUT ARGUMENTS")]
pub struct InputArg {
    #[clap(long, short, display_order = 1, default_value_t = String::from("./data.toml"))]
    data_file_path: String,
}

// Modulus + Result + Flags + Data
// M0 M1 R0 R1 R2 R3 F00 F01 F02 F03 F10 F11 F12 F13 D00 D01 D02 D03 D10 D11 D12 D13 D20 D21 D22 D23
const DATA_NUM: usize = 3;
const VALUE_NUM: usize = 2;
const COEFF_LEVEL: usize = 2;
const COEFF_DEGREE: usize = 4096;
const MODULUS_NUM: usize = COEFF_LEVEL;
const FLAG_NUM: usize = DATA_NUM - 1;
const FLAG_LEN: usize = VALUE_NUM * COEFF_LEVEL;
const DATA_LEN: usize = FLAG_LEN;
const DATA_START: usize = MODULUS_NUM + DATA_LEN + FLAG_NUM * FLAG_LEN;
const DATA_END: usize = DATA_START + DATA_NUM * DATA_LEN;
const RESULT_START: usize = MODULUS_NUM;
const RESULT_END: usize = RESULT_START + DATA_LEN;
const FLAG_START: usize = RESULT_END;

const STATE_WIDTH: usize = DATA_END;
const STATE_LENGTH: usize = COEFF_DEGREE;

pub struct PublicInputs {
    pub result: [[Vec<BaseElement>; COEFF_LEVEL]; VALUE_NUM],
}

impl Serializable for PublicInputs {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write(self.result.to_vec());
    }
}

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub result: [[Vec<u64>; COEFF_LEVEL]; VALUE_NUM],
    pub proof: String,
}

impl ::std::default::Default for Data {
    fn default() -> Self {
        Self {
            result: Default::default(),
            proof: Default::default(),
        }
    }
}

pub fn from_data(data: Data) -> (PublicInputs, Vec<u8>) {
    let mut result: [[Vec<BaseElement>; COEFF_LEVEL]; VALUE_NUM] = Default::default();
    for i in 0..VALUE_NUM {
        for j in 0..COEFF_LEVEL {
            result[i][j] = data.result[i][j]
                .iter()
                .map(|x| BaseElement::from(*x))
                .collect();
        }
    }
    (PublicInputs { result }, decode(data.proof).unwrap())
}

pub fn to_data(proof: Vec<u8>, public_input: PublicInputs) -> Data {
    let mut result: [[Vec<u64>; COEFF_LEVEL]; VALUE_NUM] = Default::default();
    for i in 0..VALUE_NUM {
        for j in 0..COEFF_LEVEL {
            result[i][j] = public_input.result[i][j]
                .iter()
                .map(|x| x.to_string().parse().unwrap())
                .collect();
        }
    }
    Data {
        result,
        proof: encode(proof),
    }
}

pub type TraceType = TraceTable<BaseElement>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct CustomData {
    pub modulus: Vec<u64>,
    pub values: [[[Vec<u64>; COEFF_LEVEL]; VALUE_NUM]; DATA_NUM],
}

impl ::std::default::Default for CustomData {
    fn default() -> Self {
        Self {
            modulus: Default::default(),
            values: Default::default(),
        }
    }
}

pub fn build_trace(arg: &InputArg) -> TraceType {
    let data: CustomData = confy::load_path(&arg.data_file_path).unwrap();
    let mut trace = TraceTable::new(STATE_WIDTH, STATE_LENGTH);

    trace.fill(
        |state| {
            for i in 0..MODULUS_NUM {
                state[i] = BaseElement::from(data.modulus[i]);
            }

            for i in DATA_START..DATA_END {
                let idx = i - DATA_START;
                let d_idx = idx / DATA_LEN;
                let v_idx = idx / COEFF_LEVEL % VALUE_NUM;
                let l_idx = idx % COEFF_LEVEL;
                state[i] = BaseElement::from(data.values[d_idx][v_idx][l_idx][0]);
            }

            for i in RESULT_START..RESULT_END {
                let idx = i - RESULT_START;
                let l_idx = idx % COEFF_LEVEL;
                let offset = i + FLAG_NUM * FLAG_LEN + DATA_LEN;
                let d1 = state[offset];
                let d2 = state[offset + DATA_LEN];
                let d3 = state[offset + 2 * DATA_LEN];
                let m = state[l_idx];
                let r1 = d1 + d2;
                if r1.is_greater(&m) {
                    state[FLAG_START + idx] = BaseElement::ONE;
                } else {
                    state[FLAG_START + idx] = BaseElement::ZERO;
                }
                if (r1 - state[FLAG_START + idx] * m).is_greater(&d3) {
                    state[FLAG_START + FLAG_LEN + idx] = BaseElement::ZERO;
                } else {
                    state[FLAG_START + FLAG_LEN + idx] = BaseElement::ONE;
                }

                state[i] = (r1 - state[FLAG_START + idx] * m)
                    + state[FLAG_START + FLAG_LEN + idx] * m
                    - d3;
            }

            for i in DATA_START..DATA_END {
                let idx = i - DATA_START;
                let d_idx = idx / DATA_LEN;
                let v_idx = idx / COEFF_LEVEL % VALUE_NUM;
                let l_idx = idx % COEFF_LEVEL;
                state[i] = BaseElement::from(data.values[d_idx][v_idx][l_idx][1]);
            }

            for i in RESULT_START..RESULT_END {
                let idx = i - RESULT_START;
                let l_idx = idx % COEFF_LEVEL;
                let offset = i + FLAG_NUM * FLAG_LEN + DATA_LEN;
                let d1 = state[offset];
                let d2 = state[offset + DATA_LEN];
                let d3 = state[offset + 2 * DATA_LEN];
                let m = state[l_idx];
                let r1 = d1 + d2;
                if r1.is_greater(&m) {
                    state[FLAG_START + idx] = BaseElement::ONE;
                } else {
                    state[FLAG_START + idx] = BaseElement::ZERO;
                }
                if (r1 - state[FLAG_START + idx] * m).is_greater(&d3) {
                    state[FLAG_START + FLAG_LEN + idx] = BaseElement::ZERO;
                } else {
                    state[FLAG_START + FLAG_LEN + idx] = BaseElement::ONE;
                }

                // println!(
                //     "fill state[{}] = {} - {} * {} + {} * {} - {} = {}",
                //     i,
                //     r1,
                //     state[FLAG_START + idx],
                //     m,
                //     state[FLAG_START + FLAG_LEN + idx],
                //     m,
                //     d3,
                //     (r1 - state[FLAG_START + idx] * m) + state[FLAG_START + FLAG_LEN + idx] * m
                //         - d3,
                // );
            }
        },
        |last_step, state| {
            for i in RESULT_START..RESULT_END {
                let idx = i - RESULT_START;
                let l_idx = idx % COEFF_LEVEL;
                let offset = i + FLAG_NUM * FLAG_LEN + DATA_LEN;
                let d1 = state[offset];
                let d2 = state[offset + DATA_LEN];
                let d3 = state[offset + 2 * DATA_LEN];
                let m = state[l_idx];
                let r1 = d1 + d2;

                state[i] = (r1 - state[FLAG_START + idx] * m)
                    + state[FLAG_START + FLAG_LEN + idx] * m
                    - d3;

                // println!(
                //     "update start state[{}] = {} - {} * {} + {} * {} - {} = {}",
                //     i,
                //     r1,
                //     state[FLAG_START + idx],
                //     m,
                //     state[FLAG_START + FLAG_LEN + idx],
                //     m,
                //     d3,
                //     state[i],
                // );
            }

            for i in DATA_START..DATA_END {
                let idx = i - DATA_START;
                let d_idx = idx / DATA_LEN;
                let v_idx = idx / COEFF_LEVEL % VALUE_NUM;
                let l_idx = idx % COEFF_LEVEL;
                state[i] = BaseElement::from(
                    data.values[d_idx][v_idx][l_idx][(last_step + 2) % COEFF_DEGREE],
                );
            }

            for i in RESULT_START..RESULT_END {
                let idx = i - RESULT_START;
                let l_idx = idx % COEFF_LEVEL;
                let offset = i + FLAG_NUM * FLAG_LEN + DATA_LEN;
                let d1 = state[offset];
                let d2 = state[offset + DATA_LEN];
                let d3 = state[offset + 2 * DATA_LEN];
                let m = state[l_idx];
                let r1 = d1 + d2;
                if r1.is_greater(&m) {
                    state[FLAG_START + idx] = BaseElement::ONE;
                } else {
                    state[FLAG_START + idx] = BaseElement::ZERO;
                }
                if (r1 - state[FLAG_START + idx] * m).is_greater(&d3) {
                    state[FLAG_START + FLAG_LEN + idx] = BaseElement::ZERO;
                } else {
                    state[FLAG_START + FLAG_LEN + idx] = BaseElement::ONE;
                }

                // println!(
                //     "update end state[{}] = {} - {} * {} + {} * {} - {} = {}",
                //     i,
                //     r1,
                //     state[FLAG_START + idx],
                //     m,
                //     state[FLAG_START + FLAG_LEN + idx],
                //     m,
                //     d3,
                //     (r1 - state[FLAG_START + idx] * m) + state[FLAG_START + FLAG_LEN + idx] * m
                //         - d3,
                // );
            }
        },
    );
    trace
}

pub fn get_pub_inputs(trace: &TraceType) -> PublicInputs {
    // [[Vec<BaseElement>; COEFF_LEVEL]; VALUE_NUM]
    PublicInputs {
        result: [
            [
                trace.get_column(0 + COEFF_LEVEL).to_vec(),
                trace.get_column(1 + COEFF_LEVEL).to_vec(),
            ],
            [
                trace.get_column(2 + COEFF_LEVEL).to_vec(),
                trace.get_column(3 + COEFF_LEVEL).to_vec(),
            ],
        ],
    }
}

pub struct FreshAir {
    context: AirContext<BaseElement>,
    result: [[Vec<BaseElement>; COEFF_LEVEL]; VALUE_NUM],
}

impl Air for FreshAir {
    type BaseField = BaseElement;
    type PublicInputs = PublicInputs;

    fn new(trace_info: TraceInfo, pub_inputs: PublicInputs, options: ProofOptions) -> Self {
        let degrees = vec![TransitionConstraintDegree::new(2); DATA_LEN];
        let num_assertions = DATA_LEN * 2;

        FreshAir {
            context: AirContext::new(trace_info, degrees, num_assertions, options),
            result: pub_inputs.result,
        }
    }

    fn context(&self) -> &AirContext<Self::BaseField> {
        &self.context
    }

    fn evaluate_transition<E: FieldElement + From<Self::BaseField>>(
        &self,
        frame: &EvaluationFrame<E>,
        _periodic_values: &[E],
        result: &mut [E],
    ) {
        let current = frame.current();
        let next = frame.next();

        for i in RESULT_START..RESULT_END {
            let idx = i - RESULT_START;
            let l_idx = idx % COEFF_LEVEL;
            let offset = i + FLAG_NUM * FLAG_LEN + DATA_LEN;
            let d1 = current[offset];
            let d2 = current[offset + DATA_LEN];
            let d3 = current[offset + 2 * DATA_LEN];
            let m = current[l_idx];
            let r1 = d1 + d2;

            let ret = (r1 - current[FLAG_START + idx] * m)
                + current[FLAG_START + FLAG_LEN + idx] * m
                - d3;
            result[idx] = next[i] - ret;
            // println!(
            //     "evaluate_transition ret:{} next[{}]:{} result[{}]:{}",
            //     ret, i, next[i], idx, result[idx]
            // );
        }
    }

    // [[Vec<BaseElement>; COEFF_LEVEL]; VALUE_NUM],
    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let last = self.trace_length() - 1;
        vec![
            Assertion::single(RESULT_START, 0, self.result[0][0][0]),
            Assertion::single(RESULT_START + 1, 0, self.result[0][1][0]),
            Assertion::single(RESULT_START + 2, 0, self.result[1][0][0]),
            Assertion::single(RESULT_START + 3, 0, self.result[1][1][0]),
            Assertion::single(RESULT_START, last, self.result[0][0][last]),
            Assertion::single(RESULT_START + 1, last, self.result[0][1][last]),
            Assertion::single(RESULT_START + 2, last, self.result[1][0][last]),
            Assertion::single(RESULT_START + 3, last, self.result[1][1][last]),
        ]
    }
}
