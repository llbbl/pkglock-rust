## Installation

### **Installing with Cargo**
You can easily install the `pkglock` utility using Cargo, Rust's package manager. Run the following command:

```bash
cargo install pkglock
```

This command will install the pkglock binary in the Cargo bin directory. 

No Cargo? Get [Rustup](https://rustup.rs/). 

## Usage
To use the pkglock utility, run the following command:

```bash
pkglock --local | --remote
```

### **Configuration**
Set up your `pkg.config.json` with the local and remote URLs necessary for your operation. The configuration file should ideally be located in the same directory as your `package-lock.json`.

`// pkg.config.json`

```json
{
  "local": "http://localhost:4873",
  "remote": "https://registry.npmjs.org"
}
```

### Running the Utility
Execute the utility from the command line, providing relevant options to match your needs.

Example: pkglock --local (switch to local NPM registry)

Example: pkglock --remote (switch to remote NPM registry)

You will want to remove the NPM version of pkglock if you have it installed globally.

### Troubleshooting 
#### Ensuring the Cargo Bin Directory is in Your PATH
To execute the pkglock utility effortlessly from any location in the terminal, ensure that the Cargo bin directory is included in your system’s PATH.

For Unix-like systems (Linux/macOS):

Open your terminal.

Add the following line to your profile script file (.bash_profile, .bashrc, .zshrc, etc.):

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Reload the profile script file to apply the changes:
```bash
source ~/.bash_profile
```

#### For Windows:
Open the Start menu, search for "Environment Variables," and choose "Edit the system environment variables."

In the System Properties window, click the "Environment Variables" button.

In the System Variables section, find and edit the Path variable to include the Cargo bin directory path:

```
C:\Users\<YourUsername>\.cargo\bin
```
Click OK to save the changes, and close the remaining windows.

Once the PATH is correctly configured, you should be able to run the pkglock utility directly from the terminal, regardless of your current directory.


### Why Use pkglock?

`npm` is slow because of so many network requests to the public internet needed to fill up your node_modules. 

A good way to speed it up is to use a local npm registry. However, switching between local and remote registries is a pain. This utility makes it easy to switch between local and remote registries.

Check out [Verdaccio](https://verdaccio.org), it is a lightweight, open-source private npm proxy registry that is highly beneficial in improving the efficiency and speed of your npm installations.

pkglock was rewritten in Rust to avoid the whole transpiling to CommonJS and ESM issue.

