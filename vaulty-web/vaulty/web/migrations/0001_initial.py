# Generated by Django 3.0.3 on 2020-05-25 21:49

import django.contrib.postgres.fields
from django.db import migrations, models
import django.db.models.deletion


class Migration(migrations.Migration):

    initial = True

    dependencies = [
    ]

    operations = [
        migrations.CreateModel(
            name='Address',
            fields=[
                ('id', models.AutoField(auto_created=True, primary_key=True, serialize=False, verbose_name='ID')),
                ('address', models.CharField(max_length=255)),
                ('is_active', models.BooleanField()),
                ('email_quota', models.IntegerField()),
                ('num_received', models.IntegerField(default=0)),
                ('max_email_size', models.IntegerField()),
                ('storage_quota', models.BigIntegerField()),
                ('storage_used', models.BigIntegerField(default=0)),
                ('last_renewal_time', models.DateTimeField()),
                ('storage_backend', models.CharField(choices=[('dropbox', 'Dropbox'), ('gdrive', 'Gdrive'), ('s3', 'S3')], max_length=30)),
                ('storage_token', models.TextField()),
                ('storage_path', models.TextField()),
                ('is_whitelist_enabled', models.BooleanField()),
                ('whitelist', django.contrib.postgres.fields.ArrayField(base_field=models.TextField(), size=None)),
                ('last_update_time', models.DateTimeField(auto_now=True)),
                ('creation_time', models.DateTimeField(auto_now_add=True)),
            ],
            options={
                'db_table': 'vaulty_addresses',
            },
        ),
        migrations.CreateModel(
            name='Alias',
            fields=[
                ('id', models.AutoField(auto_created=True, primary_key=True, serialize=False, verbose_name='ID')),
                ('is_active', models.BooleanField()),
                ('alias', models.TextField()),
                ('dest', models.TextField()),
            ],
            options={
                'db_table': 'vaulty_aliases',
            },
        ),
        migrations.CreateModel(
            name='Email',
            fields=[
                ('id', models.UUIDField(editable=False, primary_key=True, serialize=False, unique=True)),
                ('message_id', models.TextField(null=True)),
                ('num_attachments', models.IntegerField()),
                ('total_size', models.IntegerField()),
                ('status', models.BooleanField(default=True)),
                ('error_msg', models.TextField(null=True)),
                ('last_update_time', models.DateTimeField(auto_now=True)),
                ('creation_time', models.DateTimeField(auto_now_add=True)),
                ('address_id', models.ForeignKey(on_delete=django.db.models.deletion.CASCADE, to='web.Address')),
            ],
            options={
                'db_table': 'vaulty_emails',
            },
        ),
        migrations.CreateModel(
            name='User',
            fields=[
                ('id', models.AutoField(auto_created=True, primary_key=True, serialize=False, verbose_name='ID')),
                ('email_address', models.CharField(max_length=255)),
                ('password', models.CharField(max_length=255)),
                ('is_subscribed', models.BooleanField()),
                ('payment_token', models.TextField(null=True)),
                ('last_update_time', models.DateTimeField(auto_now=True)),
                ('creation_time', models.DateTimeField(auto_now_add=True)),
            ],
            options={
                'db_table': 'vaulty_users',
            },
        ),
        migrations.CreateModel(
            name='Log',
            fields=[
                ('id', models.AutoField(auto_created=True, primary_key=True, serialize=False, verbose_name='ID')),
                ('msg', models.TextField()),
                ('log_level', models.IntegerField()),
                ('creation_time', models.DateTimeField(auto_now_add=True)),
                ('email_id', models.ForeignKey(null=True, on_delete=django.db.models.deletion.CASCADE, to='web.Email')),
            ],
            options={
                'db_table': 'vaulty_logs',
            },
        ),
        migrations.AddField(
            model_name='email',
            name='user_id',
            field=models.ForeignKey(on_delete=django.db.models.deletion.CASCADE, to='web.User'),
        ),
        migrations.AddField(
            model_name='address',
            name='user_id',
            field=models.ForeignKey(null=True, on_delete=django.db.models.deletion.SET_NULL, to='web.User'),
        ),
    ]