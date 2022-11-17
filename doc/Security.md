# Security

This solution has a lot of security flaws. Let's review them.

## Lying peer

We never check if a peer is telling the truth. We could use a quorum of many
peers having the same information to ensure any liar will be discarded (and even
flagged as shady).

## 51% attack

Even if we check the correctness of information using quorum, we can't prevent
an attacker to own the quorum by having enough lying peers. Hence, can't always
prevent this kind of attack by using peers approval.

## Sybil attack

An attacker could generate an extreme amount of peer, and then flood the network
with false information. The entire network will, then, be corrupted.\
See "#Index poisoning attack" below.

## Index poisoning attack

As there is no check on who share what, one could flood the peers with false
information. In our system, it's possible to impersonate everyone (via the peer
id), and then declare to share many files. If spread to enough peers, the
network will think a user do share almost any known files (there's only 4
billions). The real peer, who was impersonated, might get spammed/ddos by all
peers wanted files from him.


