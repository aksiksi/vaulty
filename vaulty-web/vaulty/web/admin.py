from django.contrib import admin
from django.contrib.auth.admin import UserAdmin

from .models import Address, Alias, Attachment, Mail, User, LaunchMailingList


class AddressAdmin(admin.ModelAdmin):
    date_hierarchy = "creation_time"
    list_display = (
        "user", "address", "is_active", "email_quota",
        "storage_quota", "last_renewal_time", "storage_backend",
        "storage_path", "is_whitelist_enabled", "creation_time",
    )
    list_filter = ("is_active", "is_whitelist_enabled")


class MailAdmin(admin.ModelAdmin):
    list_display = (
        "user", "address", "message_id", "num_attachments",
        "total_size", "status", "creation_time",
    )
    list_filter = ("status", )


class AttachmentAdmin(admin.ModelAdmin):
    list_display = (
        "mail", "size", "index", "status", "error_msg", "creation_time",
    )
    list_filter = ("status", )


class AliasAdmin(admin.ModelAdmin):
    list_display = ("alias", "dest", "is_active")
    list_filter = ("is_active", )


class AliasAdmin(admin.ModelAdmin):
    list_display = ("alias", "dest", "is_active")
    list_filter = ("is_active", )


class LaunchMailingListAdmin(admin.ModelAdmin):
    date_hierarchy = "creation_time"


# Register models in admin
admin.site.register(User, UserAdmin)
admin.site.register(Address, AddressAdmin)
admin.site.register(Mail, MailAdmin)
admin.site.register(Attachment, AttachmentAdmin)
admin.site.register(Alias, AliasAdmin)
admin.site.register(LaunchMailingList, LaunchMailingListAdmin)
