# STARK + HE(Homomoriphic Encryption)

This repo is a test project which combins STARK and HE(Homomoriphic Encryption) technology. 

The AIR will compute `a + b - c`, while `a`,`b`,`c` are all cipher text which are produced
by companion project(<https://github.com/Vesnica/lattigo_cobra.git>).

The computed result is also a cipher text, which should send back to companion project to decrypt and
get the plain text result. 
