from django.core.mail import send_mail


def launch_mailing_list_confirmation(request, form):
    # Send an email confirmation back to the user
    email_address = form.cleaned_data["email_address"]

    send_mail(
        "Vaulty: Launch List Confirmation",
        ("Thank you for expressing interest in Vaulty!\n\n"
         "We will update you once Vaulty is ready for business :)"),
        "noreply@vaulty.net",
        [email_address],
        fail_silently=False,
    )
