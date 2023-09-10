# Cloudformation Updater (cfnupd)

A CLI tool written in rust to allow a quick and easy way to update your existing Cloudformation Stacks. It allows you to target the stack which you want to modify, perform the modifications of the template/parameters locally and then launch the update and provide feedback its success/failure. Finally it also provides the ability to save the modified artifacts for future use.

**Notes:**

- This is **NOT** a full replacement of the AWS CLI nor the AWS SDKs, this is a wrapper on top of it to speed up minor changes on the existing deployed stacks. 
- This is **NOT** the recommended approach on working with production AWS Cloudformation Stacks. This tool is primarily targeted on minor updated of dev/concept Cloudformation stacks that are not yet integrated on a full git repository with a CICD in place.

![cli-output](.docs/images/cli-output.png?raw=true "sample cli output")

## Supported Platforms

Currently the tool provides binaries for the following platforms:

- Linux
- MacOS (Not Tested)

Windows is not supported as there is not an out of the box cli text editor to use. On Windows it is recommended to install WSL and use `cfnupd` as a Linux binary. 

## How to install

### Prebuilt binaries

Download the appropriate version for your OS for the Github Releases Page and run the following command
``` bash
chmod 775 cfnupd-<version>-<os-architecture> 
sudo mv cfnupd-<version>-<os-architecture> /usr/local/bin/cfnupd
```

In case you face issues with the prebuilt binaries it is recommended to build from source.

### Build from source

1. Download and install ![rust](https://www.rust-lang.org/tools/install) on your system.
2. Clone the repository 
3. Run `make install`

## Prerequisites

### AWS Credentials

The cli requires connectivity to your AWS accounts which means either configuring the aws cli and providing the aws credentials file or populating the required environment variables for connectivity to AWS. You can find more information here: https://docs.aws.amazon.com/cli/latest/userguide/cli-configure-envvars.html

### Editor

The tool requires the modification of local files (template & parameter file) this is implemented by utilizing a cli file editor that exists currently on your system. The decision of which editor to use is performed accordingly:

1. First the environment variable `EDITOR` is checked.
2. If the environment variable does not exists, the tool check that the file in the location `/.config/cfnupd/config.toml` exists and has a value populated for `EDITOR`
3. If the above true are not correct then the file `/.config/cfnupd/config.toml` is created and sets the `EDITOR` parameter equal to `nano` (as it is ubiquitous on all systems). The user can then further modify that file to provide the preferred editor. 



## How to use

You can use the cli tool by running `cfnupd` on your command line and providing the following input parameters:

| Argument                            | Description                                                  | Default Value                   | Example Value |
| ----------------------------------- | ------------------------------------------------------------ | ------------------------------- | ------------- |
| `--stack-name` (`-s`)               | The name of the stack you want to update                     | N/A                             | `foo`         |
| `--region` (`-r`)                   | The region in which the AWS Cloudformation stack you want to update exists | Value retrieved from AWS config | `eu-west-1`   |
| `--artifacts-to-current-dir` (`-a`) | Whether or not to save the updated artifacts to the current directory. If not specified the used gets a prompt after the modification and the update occurs. (`true`/`false`) | N/A                             | `true`        |
| `--capabilities` (`-c`)             | Provide the necessary Cloudformation capabilities required for the update to be performed (`CapabilityIam`/`CapabilityNamedIam`/`CapabilityAutoExpand`). If not provided and the update requires any of this capabilities then the update will fail. | N/A                             | N/A           |
| `--verbose` (`-v`)                  | Whether or not to print verbose logs on the stdout. To be used only for debug purposes | N/A                             | N/A           |
| `--editor` (`-e`)                   | Provide the editor you want to use for modifying the CFN artifacts. If provided it will also modify the `config.toml` file with the selected editor for future calls. | N/A                             | `vim`         |

example command:

``` bash 
cfnupd -s my-stack-name -r eu-west-1 -c
```



## How it works

When the command gets triggered it downloads the Cloudformation template file (yaml) and the Cloudformation parameters file (json) in a precreated tmp directory. It then prompts the user to modify those files by opening the configured text editor on the terminal for each file respectively. After the user performs the modifications and saves the files it then performs a cloudformation update on the stack. After the update is successful it prompts the user on whether or not to save the modified artifacts on the current directory.



## Expected Behavior

When the files are not modified by the user then the cli will fail as there are not updates to be performed.



## Todo

- add tests




## License
Licensed under the Apache License, Version 2.0 ([LICENSE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0).



## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be licensed as above, without any additional terms or conditions.