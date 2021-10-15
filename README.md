# iptop
Linux IP traffic bandwidth per process monitoring tool

## Run by non-root user

`iptop` requires `cap_net_raw` capability to be run by non-root user.

```
sudo setcap "cap_net_raw+pe" target/debug/iptop
```