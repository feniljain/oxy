#include <stdio.h>
#include <stdlib.h>

struct Node {
  int data;
  struct Node *next;
  struct Node *prev;
};

void push(struct Node **head_ref, int new_data) {
  // allocate a new node
  struct Node *new_node = (struct Node *)malloc(sizeof(struct Node));
  // put in data
  new_node->data = new_data;
  // Make next of new Node as head and prev as NULL
  new_node->next = *head_ref;
  new_node->prev = NULL;
  // Make prev of old head to point to new node
  if ((*head_ref) != NULL)
    (*head_ref)->prev = new_node;
  // Change head ref to point to new node
  *head_ref = new_node;
}

void insertAfter(struct Node *prev_node, int new_data) {
  if (prev_node == NULL) {
    printf("previous node cannot be NULL");
    return;
  }

  // allocate new node
  struct Node *new_node = (struct Node *)malloc(sizeof(struct Node));
  // fill in data
  new_node->data = new_data;
  // make new_node -> prev to prev_node
  new_node->prev = prev_node;
  // make new_ndoe -> next to prev_node -> next
  new_node->next = prev_node->next;
  // make next_node -> prev to new_node
  prev_node->next->prev = new_node;
  // make prev_node -> next to new_node
  prev_node->next = new_node;
}

void append(struct Node **head_ref, int new_data) {
  // allocate new node
  struct Node *new_node = (struct Node *)malloc(sizeof(struct Node));
  // fill in data
  new_node->data = new_data;

  // assign NULL to new_node -> next
  new_node->next = NULL;

  if (*head_ref == NULL) {
    *head_ref = new_node;
    return;
  }

  // transverse to last node
  struct Node *end_node = *head_ref;
  while (end_node->next != NULL) {
    end_node = end_node->next;
  }

  // assign new_node to last_node -> next
  end_node->next = new_node;
  // assign last_node to new_node -> prev
  new_node->prev = end_node;
}

void insertBefore(struct Node *curr_node, int new_data) {

  if (curr_node == NULL) {
    printf("current node cannot be NULL");
    return;
  }

  // allocate new node
  struct Node *new_node = (struct Node *)malloc(sizeof(struct Node));
  // fill in data
  new_node->data = new_data;

  // assign curr_node to new_node -> next
  new_node->next = curr_node;
  // assign curr_node -> prev to new_node -> prev
  new_node->prev = curr_node->prev;
  // assign new_node to curr_node -> prev -> next
  curr_node->prev->next = new_node;
  // assign new_node to curr_node -> prev
  curr_node->prev = new_node;
}

/* void printList(struct Node *head_ref) { */
/*   struct Node *itr = head_ref; */
/*   while (itr->next != NULL) { */
/*     printf("%d", itr->data); */
/*     itr = itr->next; */
/*   } */
/* } */

// This function prints contents of linked list starting
// from the given node
void printList(struct Node *node) {
  struct Node *last;
  printf("\nTraversal in forward direction \n");
  while (node != NULL) {
    printf(" %d ", node->data);
    last = node;
    node = node->next;
  }

  printf("\nTraversal in reverse direction \n");
  while (last != NULL) {
    printf(" %d ", last->data);
    last = last->prev;
  }
}

/* Driver program to test above functions*/
int main() {
  /* Start with the empty list */
  struct Node *head = NULL;

  // Insert 6. So linked list becomes 6->NULL
  append(&head, 6);

  // Insert 7 at the beginning. So linked list becomes
  // 7->6->NULL
  push(&head, 7);
  printList(head);

  // Insert 1 at the beginning. So linked list becomes
  // 1->7->6->NULL
  push(&head, 1);

  // Insert 4 at the end. So linked list becomes
  // 1->7->6->4->NULL
  append(&head, 4);

  // Insert 8, after 7. So linked list becomes
  // 1->7->8->6->4->NULL
  insertAfter(head->next, 8);

  printf("Created DLL is: ");
  printList(head);

  getchar();
  return 0;
}
