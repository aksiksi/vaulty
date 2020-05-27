from django.contrib.auth.models import AbstractUser
from django.contrib.postgres.fields import ArrayField
from django.db import models


class User(AbstractUser):
    class Meta:
        db_table = "vaulty_users"

    is_subscribed = models.BooleanField()
    payment_token = models.TextField(null=True)
    last_update_time = models.DateTimeField(auto_now=True)


class Address(models.Model):
    class Meta:
        db_table = "vaulty_addresses"

    constraints = [
        models.UniqueConstraint(
            fields=["address", "is_active"],
            name="Only one address unique active address."
        ),
    ]

    class StorageBackend(models.TextChoices):
        DROPBOX = 'dropbox'
        GDRIVE = 'gdrive'
        S3 = 's3'

    # TODO: Do we want this to cascade instead?
    user = models.ForeignKey(User, models.SET_NULL, null=True)
    address = models.CharField(max_length=255)
    is_active = models.BooleanField()

    # Max number of emails this address can receive
    email_quota = models.IntegerField()

    # Number of emails this address has received in this renewal period
    num_received = models.IntegerField(default=0)

    # Max email size for this address
    max_email_size = models.IntegerField()

    # Max storage quota in renewal period, in bytes
    storage_quota = models.BigIntegerField()

    # Storage used in renewal period, in bytes
    storage_used = models.BigIntegerField(default=0)
    last_renewal_time = models.DateTimeField()
    storage_backend = models.CharField(max_length=30, choices=StorageBackend.choices)
    storage_token = models.TextField()

    # Path to store data (in valid backend format)
    storage_path = models.TextField()

    # Sender whitelisting
    is_whitelist_enabled = models.BooleanField()
    whitelist = ArrayField(models.TextField())

    last_update_time = models.DateTimeField(auto_now=True)
    creation_time = models.DateTimeField(auto_now_add=True)


class Mail(models.Model):
    class Meta:
        db_table = "vaulty_mail"

    id = models.UUIDField(primary_key=True, unique=True, editable=False)
    user = models.ForeignKey(User, models.CASCADE)
    address = models.ForeignKey(Address, models.CASCADE)
    message_id = models.TextField(null=True) # Standard MIME Message-ID
    num_attachments = models.IntegerField()
    total_size = models.IntegerField()

    # Email processed successfully by default
    status = models.BooleanField(default=True)
    error_msg = models.TextField(null=True)
    last_update_time = models.DateTimeField(auto_now=True)
    creation_time = models.DateTimeField(auto_now_add=True)


class Log(models.Model):
    class Meta:
        db_table = "vaulty_logs"

    mail = models.ForeignKey(Mail, models.CASCADE, null=True)
    msg = models.TextField()
    log_level = models.IntegerField()
    creation_time = models.DateTimeField(auto_now_add=True)


class Alias(models.Model):
    class Meta:
        db_table = "vaulty_aliases"

    is_active = models.BooleanField()
    alias = models.TextField()

    # Email will be forwarded to this address or local user
    dest = models.TextField()


class LaunchMailingList(models.Model):
    """Tracks users who signed up for launch mailing list."""
    email_address = models.CharField(max_length=255)
    creation_time = models.DateTimeField(auto_now_add=True)
