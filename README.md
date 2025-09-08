jjx is jj with (partial) hooks support.

```
$ cargo install --git https://github.com/avamsi/jjx --locked
```

Example https://github.com/j178/prek integration:

```toml
[x.hooks]
	post-commit = [
		'sh', '-c', '''
		if [ -f "$(jj root)/.pre-commit-config.yaml" ] \
		&& [ "$(jj show --template='self.empty()' --no-patch)" = "true" ]; then
			uvx prek --last-commit
		fi
		''']
	post-squash = [
		'sh', '-c', '''
		if [ -f "$(jj root)/.pre-commit-config.yaml" ] \
		&& [ "$(jj show --template='self.empty()' --no-patch)" = "true" ]; then
			uvx prek --last-commit
		fi
		''']
```
