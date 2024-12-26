# arp

## Usage

```text
arp [-vn]  [<HW>] [-i <if>] [-a] [<hostname>]             <-Display ARP cache
arp [-v]          [-i <if>] -d  <host> [pub]               <-Delete ARP entry
arp [-vnD] [<HW>] [-i <if>] -f  [<filename>]            <-Add entry from file
arp [-v]   [<HW>] [-i <if>] -s  <host> <hwaddr> [temp]            <-Add entry
arp [-v]   [<HW>] [-i <if>] -Ds <host> <if> [netmask <nm>] pub          <-''-
```

## About

manipulate the system ARP cache

## Description

**Arp** manipulates or displays the kernel's IPv4 network neighbour cache. It can add entries to the table, delete one or display the current content.

**ARP** stands for Address Resolution Protocol, which is used to find the media access control address of a network neighbour for a given IPv4 Address.

## Modes

**arp** with no mode specifier will print the current content of the table. It is possible to limit the number of entries printed, by specifying an hardware address type, interface name or host address.

**arp -d address** will delete a ARP table entry. Root or netadmin privilege is required to do this. The entry is found by IP address. If a hostname is given, it will be resolved before looking up the entry in the ARP table.

**arp -s address hw_addr** is used to set up a new table entry. The format of the **hw_addr** parameter is dependent on the hardware class, but for most classes one can assume that the usual presentation can be used. For the Ethernet class, this is 6 bytes in hexadecimal, separated by colons. When adding proxy arp entries (that is those with the publish flag set) a netmask may be specified to proxy arp for entire subnets. This is not good practice, but is supported by older kernels because it can be useful. If the temp flag is not supplied entries will be permanent stored into the ARP cache. To simplify setting up entries for one of your own network interfaces, you can use the **arp -Ds address ifname** form. In that case the hardware address is taken from the interface with the specified name.

## Arguments

- **-v**, **--verbose**

    Tell the user what is going on by being verbose.

- **-n**, **--numeric**

    shows numerical addresses instead of trying to determine symbolic host, port or user names.

- **-H**, **-t**, **--hw-type** type

    When setting or reading the ARP cache, this optional parameter tells **arp** which class of entries it should check for. The default value of this parameter is **ether** (i.e. hardware code 0x01 for IEEE 802.3 10Mbps Ethernet). Other values might include network technologies such as ARCnet (**arcnet**) , PROnet (**pronet**) , AX.25 (**ax25**) and NET/ROM (**netrom**).

- **-a**

    Use alternate BSD style output format (with no fixed columns).

- **-e**

    Use default Linux style output format (with fixed columns).

- **-D**, **--use-device**

    Instead of a hw_addr, the given argument is the name of an interface. **arp** will use the MAC address of that interface for the table entry. This is usually the best option to set up a proxy ARP entry to yourself.

- **-i**, **--device** If

    Select an interface. When dumping the ARP cache only entries matching the specified interface will be printed. When setting a permanent or **temp** ARP entry this interface will be associated with the entry; if this option is not used, the kernel will guess based on the routing table. For **pub** entries the specified interface is the interface on which ARP requests will be answered.

    **NOTE**: This has to be different from the interface to which the IP datagrams will be routed. **NOTE**: As of kernel 2.2.0 it is no longer possible to set an ARP entry for an entire subnet. Linux instead does automagic proxy arp when a route exists and it is forwarding. See arp(7) for details. Also the dontpub option which is available for delete and set operations cannot be used with 2.4 and newer kernels.

- **-f**, **--file** filename

    Similar to the **-s** option, only this time the address info is taken from file **filename**. This can be used if ARP entries for a lot of hosts have to be set up. The name of the data file is very often **/etc/ethers**, but this is not official. If no filename is specified **/etc/ethers** is used as default.

    The format of the file is simple; it only contains ASCII text lines with a hostname, and a hardware address separated by whitespace. Additionally the **pub**, **temp** and **netmask** flags can be used.
