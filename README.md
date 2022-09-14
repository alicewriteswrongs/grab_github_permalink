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
