# cfnupd

docs:
https://github.com/awslabs/aws-sdk-rust/blob/main/examples/examples/cloudformation/README.md
https://docs.rs/aws-sdk-cloudformation/latest/aws_sdk_cloudformation/

https://betterprogramming.pub/building-cli-apps-in-rust-what-you-should-consider-99cdcc67710c


tmp dir https://doc.rust-lang.org/std/env/fn.temp_dir.html

todo:

- code cleanup
- proper config support with `EDITOR`
- `thiserror` for correct error handling and feedback to user
  - App errors: eyre which is a close relative of anyhow, but has a really great error-reporting story with libraries such as color-eyre
  - Library errors: I used to use thiserror but then moved on to snafu which I use for everything. snafu gives you all the advantages of this-error but with the ergonomics of anyhow or eyre
- rm `unwrap()`
- add proper tracing
- use https://docs.rs/indicatif/latest/indicatif/ for status
- testing
- release and ship as binary
- color terminal support
- windows/mac support?

- create a tmp directory to store the script/param file. in the end of the script ask the user if they want to save the files under the current directory