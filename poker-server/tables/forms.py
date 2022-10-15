
from django import forms
from .models import Table

class NewTableForm(forms.ModelForm):
    class Meta:
        model = Table
        fields = ('name',)
