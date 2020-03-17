from django import forms


class LandingEmailForm(forms.Form):
    email = forms.EmailField(
        max_length=100,
        widget=forms.EmailInput(
            attrs={
                "class": "input is-medium",
                "placeholder": "Sign up for updates",
            }
        )
    )
