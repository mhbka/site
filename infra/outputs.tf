output "instance_ipv4" {
  description = "Public IPv4 address assigned to the OCI instance."
  value       = oci_core_instance.site.public_ip
}

output "oci_instance_id" {
  description = "OCI instance OCID."
  value       = oci_core_instance.site.id
}
