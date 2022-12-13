# grab github permalink

This is a small Rust program that takes a Github permalink, like

https://github.com/torvalds/linux/blob/master/fs/autofs/dev-ioctl.c#L28-L36

and fetches the lines referred to in the `#L${start}-L${finish}` bit. In the case
of that example link it will produce:

```c
typedef int (*ioctl_fn)(struct file *, struct autofs_sb_info *,
			struct autofs_dev_ioctl *);

static int check_name(const char *name)
{
	if (!strchr(name, '/'))
		return -EINVAL;
	return 0;
}
```

## Usage

You can install this from
[crates.io](https://crates.io/crates/grab_github_permalink) or clone this repo
and build from source, there's nothing fancy going on here build-wise.

Then you can

```sh
grab_github_permalink $URL
```

and it will just spit it to STDOUT. I'd suggest piping to `pbcopy` or `xsel` or
whatever you have handy!

### Markdown mode

If you pass the `--markdown / -m` flag then it will spit out the code snippet
formatted as a Markdown code block with a link to the original. The example link
above will come out like this:

[fs/autofs/dev-ioctl.c:L28-L36](https://github.com/torvalds/linux/blob/master/fs/autofs/dev-ioctl.c#L28-L36)
```c
typedef int (*ioctl_fn)(struct file *, struct autofs_sb_info *,
			struct autofs_dev_ioctl *);

static int check_name(const char *name)
{
	if (!strchr(name, '/'))
		return -EINVAL;
	return 0;
}
```
