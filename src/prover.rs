// Copyright Vesnica
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use std::io::Write;
use std::time::Instant;

use log::debug;
use winter_air::{FieldExtension, HashFunction, ProofOptions};
use winter_math::log2;
use winter_prover::{Prover, StarkProof, Trace};

pub mod air;
use air::{build_trace, get_pub_inputs, to_data};
use air::{BaseElement, FreshAir, InputArg, PublicInputs, TraceType};

use clap::{ArgEnum, Args, Parser};

#[derive(Parser)]
#[clap(name = "prover", author, version, about, long_about = None)]
struct Cli {
    #[clap(long, short, display_order = 1, default_value_t = String::from("./stark.toml"))]
    proof_file_path: String,
    #[clap(flatten)]
    proof_options: ProofOptionsConfig,
    #[clap(flatten)]
    input_args: InputArg,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum EnumFieldExtension {
    None,
    Quadratic,
    Cubic,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum EnumHashFunction {
    BLAKE3_192,
    BLAKE3_256,
    SHA3_256,
}

#[derive(Args)]
#[clap(next_help_heading = "PROOF OPTIONS")]
struct ProofOptionsConfig {
    #[clap(long, default_value_t = 42)]
    num_queries: usize,
    #[clap(long, default_value_t = 4)]
    blowup_factor: usize,
    #[clap(long, arg_enum, default_value_t = EnumFieldExtension::None)]
    field_extension: EnumFieldExtension,
    #[clap(long, arg_enum, default_value_t = EnumHashFunction::BLAKE3_256)]
    hash_fn: EnumHashFunction,
    #[clap(long, default_value_t = 16)]
    grinding_factor: u32,
    #[clap(long, default_value_t = 8)]
    folding_factor: usize,
    #[clap(long, default_value_t = 256)]
    fri_max_remainder_size: usize,
}

fn new_proof_options(opt: &ProofOptionsConfig) -> ProofOptions {
    let field_extension = match opt.field_extension {
        EnumFieldExtension::None => FieldExtension::None,
        EnumFieldExtension::Quadratic => FieldExtension::Quadratic,
        EnumFieldExtension::Cubic => FieldExtension::Cubic,
    };
    let hash_fn = match opt.hash_fn {
        EnumHashFunction::BLAKE3_192 => HashFunction::Blake3_192,
        EnumHashFunction::BLAKE3_256 => HashFunction::Blake3_256,
        EnumHashFunction::SHA3_256 => HashFunction::Sha3_256,
    };

    ProofOptions::new(
        opt.num_queries,
        opt.blowup_factor,
        opt.grinding_factor,
        hash_fn,
        field_extension,
        opt.folding_factor,
        opt.fri_max_remainder_size,
    )
}

struct ProveOutput {
    proof: StarkProof,
    public_input: PublicInputs,
}

fn prove(cli: &Cli) -> ProveOutput {
    // generate the execution trace
    debug!(
        "Generating proof for computing a test algorithm with input_args {:?} \n\
        ---------------------",
        cli.input_args
    );

    // create a prover
    let prover = FreshProver {
        options: new_proof_options(&cli.proof_options),
    };

    // generate the execution trace
    let now = Instant::now();
    let trace = build_trace(&cli.input_args);
    let public_input = prover.get_pub_inputs(&trace);
    let trace_length = trace.length();
    debug!(
        "Generated execution trace of {} registers and 2^{} steps in {} ms",
        trace.width(),
        log2(trace_length),
        now.elapsed().as_millis()
    );

    // generate the proof
    ProveOutput {
        proof: prover.prove(trace).unwrap(),
        public_input,
    }
}

pub struct FreshProver {
    options: ProofOptions,
}

// When implementing Prover trait we set the `Air` associated type to the AIR of the
// computation we defined previously, and set the `Trace` associated type to `TraceTable`
// struct as we don't need to define a custom trace for our computation.
impl Prover for FreshProver {
    type BaseField = BaseElement;
    type Air = FreshAir;
    type Trace = TraceType;

    // Our public inputs consist of the first and last value in the execution trace.
    fn get_pub_inputs(&self, trace: &Self::Trace) -> PublicInputs {
        get_pub_inputs(trace)
    }

    fn options(&self) -> &ProofOptions {
        &self.options
    }
}

fn main() {
    env_logger::Builder::new()
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .filter_level(log::LevelFilter::Debug)
        .init();

    let cli = Cli::parse();

    let now = Instant::now();
    let output = prove(&cli);
    debug!(
        "---------------------\nProof generated in {} ms",
        now.elapsed().as_millis()
    );

    let proof_bytes = output.proof.to_bytes();
    debug!("Proof size: {:.1} KB", proof_bytes.len() as f64 / 1024f64);
    debug!("Proof security: {} bits", output.proof.security_level(true));

    let data = to_data(proof_bytes, output.public_input);
    confy::store_path(cli.proof_file_path, data).unwrap();
}
