from django.shortcuts import get_object_or_404, render, redirect
from django.contrib.auth.decorators import login_required
from django.http import HttpResponse, HttpResponseNotAllowed
from .models import TableState
from .models import Table

import poker_core_py

def latest_state(table_id, filtered_for=None):
    states = TableState.objects.filter(table_id=table_id).order_by('-id')[:1]
    if len(states):
        return states[0].data
    state_data = poker_core_py.new_game_state()
    print('made new state ------- ')
    print(state_data)
    state = TableState(table=Table.objects.get(pk=table_id), data=state_data)
    state.save()
    if filtered_for is None:
        return state.data
    return poker_core_py.filter_state(state.data, filtered_for)

def save_state(table_id, state_data):
    state = TableState(table=Table.objects.get(pk=table_id), data=state_data)
    state.save()

def index(request):
    latest_tables = Table.objects.order_by('-creation_date')[:10]
    context = {'latest_tables': latest_tables}
    return render(request, 'tables/index.html', context)

@login_required
def detail(request, table_id):
    table = get_object_or_404(Table, pk=table_id)
    if request.method != 'POST':
        return render(request, 'tables/detail.html', {'table': table})
    user_id = request.user.id
    table_id = table.id
    stack = 1000
    state = latest_state(table_id)
    new_state = poker_core_py.seat_player(state, user_id, stack)
    TableState(table=Table.objects.get(pk=table_id), data=new_state).save()
    return redirect('tables:play', table_id)

@login_required
def play(request, table_id):
    table = get_object_or_404(Table, pk=table_id)
    user = request.user
    # TODO: return page showing table and loading WASM that polls for state updates
    return render(
        request,
        'tables/play.html',
        {
            'state': latest_state(table.id, filtered_for=user.id),
            'table': table,
        }
    )

@login_required
def state(request, table_id):
    table = get_object_or_404(Table, pk=table_id)
    # TODO: ensure user is seated at table
    user = request.user
    return HttpResponse(latest_state(table.id, filtered_for=user.id))

@login_required
def method_reset(request, table_id):
    # TODO: ensure user is seated at table
    # TODO: ensure user is admin of table? in future this code should be removed, so maybe not important
    table = get_object_or_404(Table, pk=table_id)
    state = poker_core_py.devonly_reset_state(latest_state(table_id))
    save_state(table.id, state)
    return redirect('tables:play', table.id)
