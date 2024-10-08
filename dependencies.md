# Dependencies

## Cargo

To install the rust toolchain for building the project go to
[rust-lang.org](https://www.rust-lang.org/tools/install) and follow the instructions.

## Protobuf

Protobuf's CLI (protoc) is required to build this project as it is used to auto-generate rust code based on the [protobuf](/proto/keymapp.proto) of the underlying API which communicates with Keymapp.

### Windows

Install using [winget](https://learn.microsoft.com/en-us/windows/package-manager/winget/):

`winget install protobuf`

In order for tonic-build to build, it needs to know the path of protoc.
Therefore, we need to add the executable path to an environment variable using powershell:

```pwsh
[Environment]::SetEnvironmentVariable('PROTOC', '%USERPROFILE%\AppData\Local\Microsoft\WinGet\Packages\Google.Protobuf_Microsoft.Winget.Source_8wekyb3d8bbwe\bin\protoc.exe', 'User')
```

### Linux & MacOS

For Linux & MacOS you can follow Google's installation instructions:
[grpc.io](https://grpc.io/docs/protoc-installation/)
