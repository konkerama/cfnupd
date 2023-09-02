# cfnupd

docs:
https://github.com/awslabs/aws-sdk-rust/blob/main/examples/examples/cloudformation/README.md
https://docs.rs/aws-sdk-cloudformation/latest/aws_sdk_cloudformation/

https://betterprogramming.pub/building-cli-apps-in-rust-what-you-should-consider-99cdcc67710c

tmp dir https://doc.rust-lang.org/std/env/fn.temp_dir.html

todo:

- test on a more complicated cfn use-case
- unit testing

gh actions:
- run tests
- deploy binary for:
  - linux 
  - mac
  - windows

expected behavior: 
throws error if no modifications are made