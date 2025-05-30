# Pinocchio Example Programs
After dipping my feet into the great course of https://blueshift.gg/, I became interested in Pinocchio. I like the lightweight approach (especially compared to Anchor).

I couldn't find many examples, so I decided to create this repository to showcase some use cases and how they can be implemented.

But even the Vote Program showed me how many things we take for granted when working with Anchor. So much heavy lifting is done, hidden from the user.

## Vote Program
The Vote Program allows users to vote for any Name. Some hard limits aren't currently properly guarded, e.g., String length.
I implemented a custom serialization & deserialization from the binary representation for my Struct.

Compile the binary and run the tests.
```
cargo build-sbf && cargo test
```
