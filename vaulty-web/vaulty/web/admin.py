from django.contrib import admin
from django.contrib.auth.admin import UserAdmin

from .models import Address, Alias, User


class AddressAdmin(admin.ModelAdmin):
    date_hierarchy = "creation_time"
    list_display = (
        "user", "address", "is_active", "email_quota",
        "storage_quota", "last_renewal_time", "storage_backend",
        "storage_path", "is_whitelist_enabled", "creation_time",
    )
    list_filter = ("is_active", "is_whitelist_enabled")


class AliasAdmin(admin.ModelAdmin):
    list_display = ("alias", "dest", "is_active")
    list_filter = ("is_active", )


# Register models in admin
admin.site.register(Address, AddressAdmin)
admin.site.register(Alias, AliasAdmin)
admin.site.register(User, UserAdmin)
