from django import forms

from .models import LaunchMailingList


class LaunchEmailForm(forms.ModelForm):
    class Meta:
        model = LaunchMailingList
        fields = ["email"]
        widgets = {
            "email": forms.EmailInput(
                attrs={
                    "class": "input is-medium",
                    "placeholder": "Sign up for updates",
                }
            ),
        }
