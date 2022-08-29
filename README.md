# STARK + HE(Homomoriphic Encryption)

## Need patch winter_math::fields::f128::BaseElement first:
```rust
impl BaseElement {
    /// Creates a new field element from a u128 value. If the value is greater than or equal to
    /// the field modulus, modular reduction is silently performed. This function can also be used
    /// to initialize constants.
    pub const fn new(value: u128) -> Self {
        BaseElement(if value < M { value } else { value - M })
    }

    /// Must add this function
    pub fn is_greater(&self, v: &Self) -> bool {
        self.0 > v.0
    }
}
```

This repo is a test project which combins STARK and HE(Homomoriphic Encryption) technology. 

The AIR will compute `a + b - c`, while `a`,`b`,`c` are all cipher text which are produced
by companion project(<https://github.com/Vesnica/lattigo_cobra>).

The computed result is also a cipher text, which should send back to companion project to decrypt and
get the plain text result. 
