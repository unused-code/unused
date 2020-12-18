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

To update, run:

```sh
brew update
brew upgrade unused
```

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
