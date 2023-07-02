# Project Hole

Project Hole is a peer-to-peer (P2P) communication application that utilizes hole punching to enable direct communication between devices in different network environments. This allows for various use cases, such as real-time communication, file sharing, multiplayer gaming, and more, without the need for centralized servers.

## How It Works

Hole punching is a technique that enables direct connections between devices behind NAT routers or firewalls. NAT assigns private IP addresses to devices on a local network, making direct communication between devices on different networks challenging. To overcome this, devices connect to a central rendezvous server, which helps exchange public IP addresses and ports. Sending packets simultaneously to each other's public IP addresses and ports creates temporary port mappings, allowing incoming packets to reach the devices, "punching a hole" in the NAT. This facilitates direct P2P communication without relying on centralized servers.
