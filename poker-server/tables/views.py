from django.shortcuts import get_object_or_404, render
from django.http import Http404

from .models import Table

def index(request):
    latest_tables = Table.objects.order_by('-creation_date')[:10]
    context = {'latest_tables': latest_tables}
    return render(request, 'tables/index.html', context)

def detail(request, table_id):
    table = get_object_or_404(Table, pk=table_id)
    return render(request, 'tables/detail.html', {'table': table})

