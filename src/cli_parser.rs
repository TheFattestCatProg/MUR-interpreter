use std::env::Args;


pub struct CmdArgs {
    only_expand: bool,
    file_path: String
}

impl CmdArgs {
    pub fn parse(args: Args) -> Result<Self, String> {
        let mut only_expand = false;
        let mut file_path = String::from("");

        for i in args.into_iter().skip(1) {
            if i.starts_with('-') {
                let i = &i[1..];

                if i == "m" {
                    if !only_expand {
                        only_expand = true;
                    }
                    else {
                        return Err(String::from("'-m' was already used"));
                    }
                }

                continue;
            }

            if file_path.is_empty() {
                file_path = i;
            }
            else {
                return Err(String::from("File was already specified!"))
            }
        }

        Ok(CmdArgs {
            only_expand: only_expand,
            file_path: file_path
        })
    }

    pub fn only_expand(&self) -> bool {
        self.only_expand
    }

    pub fn filepath(&self) -> &String {
        &self.file_path
    }
}