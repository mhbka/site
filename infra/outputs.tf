output "instance_ipv4" {
  description = "Public IPv4 address assigned to the DigitalOcean Droplet."
  value       = digitalocean_droplet.site.ipv4_address
}

output "digitalocean_droplet_id" {
  description = "DigitalOcean Droplet ID."
  value       = digitalocean_droplet.site.id
}
