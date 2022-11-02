# Unused

![CI](https://github.com/unused-code/unused/workflows/CI/badge.svg)
![Security audit](https://github.com/unused-code/unused/workflows/Security%20audit/badge.svg)

Unused identifies unused code in Rails, Phoenix, and other types of
applications, improving developer productivity

![Screenshot of Unused Output](https://unused.codes/images/unused-output.png)

## Installing and Updating

### Homebrew (Recommended)

You can install [the formulae] via [Homebrew] with `brew tap`:

```sh
brew tap unused-code/formulae
```

Next, run:

```sh
brew install unused
```

[the formulae]: https://github.com/unused-code/formulae
[Homebrew]: http://brew.sh/

This will install `unused` and its corresponding dependencies.

#### Updating

To update, run:

```sh
brew update
brew upgrade unused
```

#### Caveats (Apple Installation on M1 and Intel CPUs)

`unused` can be installed to use a memory allocator called [mimalloc].  
In local benchmarks (which are documented in the [commit introducing mimalloc]),
it speeds up execution by a significant amount, but unfortunately runs 
into sporadic [issues with segmentation faults](https://github.com/unused-code/unused/issues/34)
on Apple M1 devices.  For this reason, `unused` by default is installed 
without mimalloc on Macs, using the stock Rust allocator instead.

If you are using a Mac with an Intel chip and you wish to use mimalloc, 
you must specifically request it:

```
brew install unused --with-mimalloc
```

[mimalloc]: https://github.com/microsoft/mimalloc
[commit introducing mimalloc]: https://github.com/unused-code/unused/commit/a206e557af47109ae7f907b89649da8a39fed932

To completely refresh your install:

```sh
brew uninstall unused
brew untap unused-code/formulae
brew tap unused-code/formulae
brew install unused --with-mimalloc
```

### Nix

There is a [Nix] expression [available in nixpkgs].

There are many ways to run `unused` with Nix, but the simplest is:

```sh
nix-shell -I nixpkgs=https://github.com/NixOS/nixpkgs/archive/nixpkgs-unstable.tar.gz -p unused --run 'unused --help'
```

[nix]: https://nixos.org
[available in nixpkgs]: https://github.com/NixOS/nixpkgs/blob/master/pkgs/development/tools/misc/unused/default.nix

### Linux

Precompiled binaries are available for [the latest release].

Download the linux-musl binary, and move the binary to somewhere within your `$PATH`.

[the latest release]: https://github.com/unused-code/unused/releases/latest

## Prerequisites

It is strongly recommended you install [Universal Ctags] to generate tags
files. Universal Ctags supports more languages and has native parsers for a
good number of them, resulting in faster tags generation time.

[Universal Ctags]: https://ctags.io/

It is also recommended that you have a tags file generated on a semi-regular
basis. Tim Pope wrote an article about [wiring up ctags generation] with a [git
hook]. thoughtbot's dotfiles also reference a [ctags git hook].

[wiring up ctags generation]: https://tbaggery.com/2011/08/08/effortless-ctags-with-git.html
[git hook]: https://git-scm.com/book/en/v2/Customizing-Git-Git-Hooks
[ctags git hook]: https://github.com/thoughtbot/dotfiles/blob/master/git_template/hooks/ctags

## Usage

From your project directory, run:

```sh
unused
```

This will generate a list of tokens and corresponding definition locations for
removal consideration.

You can see supported command-line flags with:

```sh
unused --help
```

## Troubleshooting

If you run into trouble, run

```sh
unused doctor
```

This will perform a series of simple checks to help identify obvious issues
with configuration.

## License

Copyright 2020 Josh Clayton. See the [LICENSE](LICENSE).
