from django import forms

from .models import LaunchMailingList


class LaunchEmailForm(forms.ModelForm):
    class Meta:
        model = LaunchMailingList
        fields = ["email_address"]
        widgets = {
            "email_address": forms.EmailInput(
                attrs={
                    "class": "input is-medium",
                    "placeholder": "Sign up for updates",
                }
            ),
        }
