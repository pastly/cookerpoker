from django.db import models
from django.contrib.auth.models import User

class Table(models.Model):
    owner = models.ForeignKey(User, null=True, on_delete=models.SET_NULL)
    name = models.CharField(max_length=128)
    creation_date = models.DateTimeField('date created', auto_now_add=True)

    def __str__(self):
        return f'{self.name} (owner {self.owner})'

class TableState(models.Model):
    table = models.ForeignKey(Table, null=True, on_delete=models.SET_NULL)
    data = models.TextField()
