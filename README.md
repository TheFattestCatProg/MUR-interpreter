Overview
-
***Machine with Unlimited Registers (MUR)*** is abstract model such as 
***Turing Machine*** and ***Lambda Calculus***.

**MUR** have 4 basic operations:
- Set zero `zer %R (%R = 0)`
- Add one `inc %R (%R += 1)`
- Move value from one register to other `mov %R1 %R2 (%R1 <- %R2)`
- Conditional jump `jmp %R1 %R2 @M (if %R1 == %R2 goto @M)`

This project realizes such machine in ***Rust***.
See examples.

Features:
-
- [x] Basic 4 operations
- [x] `out %R` operator
- [x] Macro

Use:
-
To run `cargo run --release -- file/path.mur`.

To view full macro expansion `cargo run --release -- file/path.mur -m`.