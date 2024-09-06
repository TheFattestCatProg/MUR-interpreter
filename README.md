Overview
-
***Machine with Unlimited Registers (MUR)*** is abstract model such as 
***Turing Machine*** and ***Lambda Calculus***.

**MUR** have 4 basic operations:
- Set zero **(zer %R)**
- Add one **(inc %R)**
- Move value from one register to other **(mov %R1 %R2)**
- Conditional jump == **(jmp %R1 %R2 @M)**

This project realizes such machine in ***Rust***.

Features:
-
- [x] Basic 4 operations
- [x] Out operator
- [ ] Custom operations