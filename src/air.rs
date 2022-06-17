// Copyright Vesnica
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use winter_air::{
    Air, AirContext, Assertion, EvaluationFrame, ProofOptions, TraceInfo,
    TransitionConstraintDegree,
};
use winter_math::FieldElement;
use winter_prover::{Trace, TraceTable};
use winter_utils::{ByteWriter, Serializable};

use base64::{decode, encode};
use clap::Args;
use serde::{Deserialize, Serialize};

pub type BaseElement = winter_math::fields::f62::BaseElement;

#[derive(Args, Debug)]
#[clap(next_help_heading = "INPUT ARGUMENTS")]
pub struct InputArg {
    #[clap(long, default_value_t = 0)]
    pub start: u64,
    #[clap(long, default_value_t = 1024)]
    pub n: usize,
}

pub struct PublicInputs {
    pub start: BaseElement,
    pub result: BaseElement,
}

impl Serializable for PublicInputs {
    fn write_into<W: ByteWriter>(&self, target: &mut W) {
        target.write(self.start);
        target.write(self.result);
    }
}

#[derive(Serialize, Deserialize)]
pub struct Data {
    pub start: String,
    pub result: String,
    pub proof: String,
}

impl ::std::default::Default for Data {
    fn default() -> Self {
        Self {
            start: "0".into(),
            result: "0".into(),
            proof: "".into(),
        }
    }
}

pub fn from_data(data: Data) -> (PublicInputs, Vec<u8>) {
    (
        PublicInputs {
            start: BaseElement::new(data.start.parse().unwrap()),
            result: BaseElement::new(data.result.parse().unwrap()),
        },
        decode(data.proof).unwrap(),
    )
}

pub fn to_data(proof: Vec<u8>, public_input: PublicInputs) -> Data {
    Data {
        start: public_input.start.to_string(),
        result: public_input.result.to_string(),
        proof: encode(proof),
    }
}

pub type TraceType = TraceTable<BaseElement>;

pub fn build_trace(arg: &InputArg) -> TraceType {
    let trace_width = 1;
    let mut trace = TraceTable::new(trace_width, arg.n);

    trace.fill(
        |state| {
            state[0] = BaseElement::new(arg.start);
        },
        |_, state| {
            state[0] = state[0].exp(3u32.into()) + BaseElement::new(42);
        },
    );

    trace
}

pub fn get_pub_inputs(trace: &TraceType) -> PublicInputs {
    let last_step = trace.length() - 1;
    PublicInputs {
        start: trace.get(0, 0),
        result: trace.get(0, last_step),
    }
}

pub struct FreshAir {
    context: AirContext<BaseElement>,
    start: BaseElement,
    result: BaseElement,
}

impl Air for FreshAir {
    type BaseField = BaseElement;
    type PublicInputs = PublicInputs;

    fn new(trace_info: TraceInfo, pub_inputs: PublicInputs, options: ProofOptions) -> Self {
        assert_eq!(1, trace_info.width());

        let degrees = vec![TransitionConstraintDegree::new(3)];
        let num_assertions = 2;

        FreshAir {
            context: AirContext::new(trace_info, degrees, num_assertions, options),
            start: pub_inputs.start,
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
        let current_state = &frame.current()[0];
        let next_state = current_state.exp(3u32.into()).add(E::from(42u32));

        result[0] = frame.next()[0] - next_state;
    }

    fn get_assertions(&self) -> Vec<Assertion<Self::BaseField>> {
        let last_step = self.trace_length() - 1;
        vec![
            Assertion::single(0, 0, self.start),
            Assertion::single(0, last_step, self.result),
        ]
    }
}
