# STARK + HE(Homomoriphic Encryption)

This repo is a test project which combins STARK and HE(Homomoriphic Encryption) technology. 

The AIR will compute `a + b - c`, while `a`,`b`,`c` are all cipher text which are produced
by lattigo(<https://github.com/tuneinsight/lattigo>) library.

The computed result is also a cipher text, which should send back to lattigo to decrypt and
get the plain text result. 
